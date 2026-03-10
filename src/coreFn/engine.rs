use crate::coreFn::strategy::strategy;
use crate::executor::{
    bybit_close::bybit_close,
    bybit_open::{BybitRead, BybitWrite, bybit_open},
    okx_close::okx_close,
    okx_open::{OkxRead, OkxWrite, okx_open},
};
use crate::model::{
    quote::Quote,
    signal::{Signal, State},
};
use crate::ws_task::{bybit_ws_task::bybit_ws_task, okx_ws_task::okx_bbo_tbt_loop};

use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use dotenvy::dotenv;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::env;
use tokio::select;
use tokio::sync::watch;
use tokio_tungstenite::{connect_async, tungstenite::Message};

pub async fn engine() -> anyhow::Result<()> {
    // let target_coin = 14.0_f64;
    let ct_mult = get_okx_ct_mult("FLOW-USDT-SWAP").await?;
    // let okx_sz = (target_coin / ct_mult).floor() as u64;
    let okx_sz = 14.to_string();
    let bybit_qty = 140.to_string();

    let mut state = State::Idle;

    // 先登录交易 ws
    let (mut okx_write, okx_read) = okx_login().await?;
    let (mut bybit_write, bybit_read) = bybit_login().await?;

    // 行情 channel
    let (okx_tx, mut okx_rx) = watch::channel::<Option<Quote>>(None);
    let (bybit_tx, mut bybit_rx) = watch::channel::<Option<Quote>>(None);

    // 连接存活状态 channel
    let (okx_alive_tx, mut okx_alive_rx) = watch::channel(true);
    let (bybit_alive_tx, mut bybit_alive_rx) = watch::channel(true);

    // 行情任务
    tokio::spawn(async move {
        let _ = okx_bbo_tbt_loop("FLOW-USDT-SWAP", okx_tx).await;
    });
    tokio::spawn(async move {
        let _ = bybit_ws_task("FLOWUSDT", bybit_tx).await;
    });

    // 交易 ws 空读守卫
    tokio::spawn(async move {
        let _ = okx_read_guard(okx_read, okx_alive_tx).await;
    });
    tokio::spawn(async move {
        let _ = bybit_read_guard(bybit_read, bybit_alive_tx).await;
    });

    let mut last_okx: Option<Quote> = None;
    let mut last_bybit: Option<Quote> = None;

    loop {
        select! {
            r = okx_rx.changed() => {
                if r.is_err() {
                    anyhow::bail!("okx quote channel closed");
                }
                last_okx = okx_rx.borrow_and_update().clone();
            }

            r = bybit_rx.changed() => {
                if r.is_err() {
                    anyhow::bail!("bybit quote channel closed");
                }
                last_bybit = bybit_rx.borrow_and_update().clone();
            }

            r = okx_alive_rx.changed() => {
                if r.is_err() {
                    anyhow::bail!("okx alive channel closed");
                }

                let alive = *okx_alive_rx.borrow_and_update();
                if !alive {
                    println!("OKX 交易 ws 已断开，开始重连...");

                    let (new_okx_write, new_okx_read) = okx_login().await?;
                    okx_write = new_okx_write;

                    let (new_okx_alive_tx, new_okx_alive_rx) = watch::channel(true);
                    okx_alive_rx = new_okx_alive_rx;

                    tokio::spawn(async move {
                        let _ = okx_read_guard(new_okx_read, new_okx_alive_tx).await;
                    });

                    println!("OKX 交易 ws 重连成功");
                }
            }

            r = bybit_alive_rx.changed() => {
                if r.is_err() {
                    anyhow::bail!("bybit alive channel closed");
                }

                let alive = *bybit_alive_rx.borrow_and_update();
                if !alive {
                    println!("Bybit 交易 ws 已断开，开始重连...");

                    let (new_bybit_write, new_bybit_read) = bybit_login().await?;
                    bybit_write = new_bybit_write;

                    let (new_bybit_alive_tx, new_bybit_alive_rx) = watch::channel(true);
                    bybit_alive_rx = new_bybit_alive_rx;

                    tokio::spawn(async move {
                        let _ = bybit_read_guard(new_bybit_read, new_bybit_alive_tx).await;
                    });

                    println!("Bybit 交易 ws 重连成功");
                }
            }
        }

        if let (Some(okx), Some(bybit)) = (&last_okx, &last_bybit) {
            if let Some(signal) = strategy(okx, bybit, ct_mult, &state) {
                match signal {
                    Signal::OpenOkxLongBybitShort => {
                        println!("触发开仓信号");
                        state = State::Opened;

                        // OKX 开空
                        if let Err(e) =
                            okx_open(&mut okx_write, "FLOW-USDT-SWAP", "buy", &okx_sz).await
                        {
                            eprintln!("OKX 开仓失败，尝试重连: {}", e);

                            let (new_okx_write, new_okx_read) = okx_login().await?;
                            okx_write = new_okx_write;

                            let (new_okx_alive_tx, new_okx_alive_rx) = watch::channel(true);
                            okx_alive_rx = new_okx_alive_rx;

                            tokio::spawn(async move {
                                let _ = okx_read_guard(new_okx_read, new_okx_alive_tx).await;
                            });

                            if let Err(e2) =
                                okx_open(&mut okx_write, "FLOW-USDT-SWAP", "sell", &okx_sz).await
                            {
                                state = State::Idle;
                                anyhow::bail!("OKX 重连后开仓仍失败: {}", e2);
                            }
                        }

                        // Bybit 开多
                        if let Err(e) =
                            bybit_open(&mut bybit_write, "FLOWUSDT", "Sell", &bybit_qty).await
                        {
                            eprintln!("Bybit 开仓失败，尝试重连: {}", e);

                            let (new_bybit_write, new_bybit_read) = bybit_login().await?;
                            bybit_write = new_bybit_write;

                            let (new_bybit_alive_tx, new_bybit_alive_rx) = watch::channel(true);
                            bybit_alive_rx = new_bybit_alive_rx;

                            tokio::spawn(async move {
                                let _ = bybit_read_guard(new_bybit_read, new_bybit_alive_tx).await;
                            });

                            if let Err(e2) =
                                bybit_open(&mut bybit_write, "FLOWUSDT", "Buy", &bybit_qty).await
                            {
                                state = State::Idle;
                                anyhow::bail!("Bybit 重连后开仓仍失败: {}", e2);
                            }
                        }

                        println!("开仓完成，当前状态: Opened");
                    }

                    Signal::Close => {
                        println!("触发平仓信号");

                        // OKX 平多
                        if let Err(e) =
                            okx_close(&mut okx_write, "FLOW-USDT-SWAP", "sell", &okx_sz).await
                        {
                            eprintln!("OKX 平仓失败，尝试重连: {}", e);

                            let (new_okx_write, new_okx_read) = okx_login().await?;
                            okx_write = new_okx_write;

                            let (new_okx_alive_tx, new_okx_alive_rx) = watch::channel(true);
                            okx_alive_rx = new_okx_alive_rx;

                            tokio::spawn(async move {
                                let _ = okx_read_guard(new_okx_read, new_okx_alive_tx).await;
                            });

                            okx_close(&mut okx_write, "FLOW-USDT-SWAP", "sell", &okx_sz).await?;
                        }

                        // Bybit 平空
                        if let Err(e) =
                            bybit_close(&mut bybit_write, "FLOWUSDT", "Buy", &bybit_qty).await
                        {
                            eprintln!("Bybit 平仓失败，尝试重连: {}", e);

                            let (new_bybit_write, new_bybit_read) = bybit_login().await?;
                            bybit_write = new_bybit_write;

                            let (new_bybit_alive_tx, new_bybit_alive_rx) = watch::channel(true);
                            bybit_alive_rx = new_bybit_alive_rx;

                            tokio::spawn(async move {
                                let _ = bybit_read_guard(new_bybit_read, new_bybit_alive_tx).await;
                            });

                            bybit_close(&mut bybit_write, "FLOWUSDT", "Buy", &bybit_qty).await?;
                        }

                        println!("平仓完成，程序结束");
                        return Ok(());
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct OkxInstResp {
    data: Vec<OkxInstData>,
}

#[derive(Deserialize)]
struct OkxInstData {
    #[serde(rename = "ctVal")]
    ct_val: String,
}

pub async fn get_okx_ct_mult(inst_id: &str) -> anyhow::Result<f64> {
    let url = format!(
        "https://www.okx.com/api/v5/public/instruments?instType=SWAP&instId={}",
        inst_id
    );
    let resp: OkxInstResp = reqwest::get(&url).await?.json().await?;

    let ct_mult = resp
        .data
        .get(0)
        .ok_or(anyhow::anyhow!("no instrument"))?
        .ct_val
        .parse::<f64>()?;

    Ok(ct_mult)
}

fn sign_okx(secret: &str, prehash: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

fn sign_bybit(secret: &str, prehash: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}

// 登陆逻辑
async fn okx_login() -> anyhow::Result<(OkxWrite, OkxRead)> {
    dotenv().ok();
    let api_key = env::var("OKX_API_KEY")?;
    let secret = env::var("OKX_SECRET_KEY")?;
    let passphrase = env::var("OKX_PASSPHRASE")?;

    let url = "wss://ws.okx.com:8443/ws/v5/private";
    let (ws, _) = connect_async(url).await?;
    println!("OKX 已连接");

    let (mut write, mut read) = ws.split();

    let ts = Utc::now().timestamp().to_string();
    let prehash = format!("{ts}GET/users/self/verify");
    let sign = sign_okx(&secret, &prehash);

    let login = serde_json::json!({
        "op":"login",
        "args":[{
            "apiKey": api_key,
            "passphrase": passphrase,
            "timestamp": ts,
            "sign": sign
        }]
    });

    write.send(Message::Text(login.to_string().into())).await?;
    println!("OKX 已发送 login");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("OKX login返回: {}", text);
            let v: serde_json::Value = serde_json::from_str(&text)?;

            if v["event"] == "login" {
                if v["code"] == "0" {
                    println!("OKX 登录成功");
                    return Ok((write, read));
                } else {
                    anyhow::bail!("OKX 登录失败: {}", text);
                }
            }
        }
    }

    anyhow::bail!("OKX 登录时连接断开")
}

async fn bybit_login() -> anyhow::Result<(BybitWrite, BybitRead)> {
    dotenv().ok();
    let api_key = env::var("BYBIT_API_KEY")?;
    let secret = env::var("BYBIT_SECRET")?;

    let url = "wss://stream.bybit.com/v5/trade";
    let (ws, _) = connect_async(url).await?;
    println!("Bybit 已连接");

    let (mut write, mut read) = ws.split();

    let expires = Utc::now().timestamp_millis() + 5000;
    let sign_payload = format!("GET/realtime{}", expires);
    let signature = sign_bybit(&secret, &sign_payload);

    let login_msg = serde_json::json!({
        "op":"auth",
        "args":[api_key, expires, signature]
    });

    write
        .send(Message::Text(login_msg.to_string().into()))
        .await?;

    println!("Bybit 已发送 auth");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("Bybit auth返回: {}", text);
            let v: serde_json::Value = serde_json::from_str(&text)?;

            if v["op"] == "auth" {
                if v["retCode"] == 0 || v["success"] == true {
                    println!("Bybit 登录成功");
                    return Ok((write, read));
                } else {
                    anyhow::bail!("Bybit 登录失败: {}", text);
                }
            }
        }
    }

    anyhow::bail!("Bybit 登录时连接断开")
}

pub async fn okx_read_guard(mut read: OkxRead, tx: watch::Sender<bool>) -> anyhow::Result<()> {
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("OKX trade msg: {}", text);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("OKX read error: {}", e);
                let _ = tx.send(false);
                anyhow::bail!("okx read error: {}", e);
            }
        }
    }

    let _ = tx.send(false);
    anyhow::bail!("okx read closed")
}

pub async fn bybit_read_guard(mut read: BybitRead, tx: watch::Sender<bool>) -> anyhow::Result<()> {
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Bybit trade msg: {}", text);
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Bybit read error: {}", e);
                let _ = tx.send(false);
                anyhow::bail!("bybit read error: {}", e);
            }
        }
    }

    let _ = tx.send(false);
    anyhow::bail!("bybit read closed")
}
