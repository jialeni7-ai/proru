use crate::executor::bybit_open::{bybit_open,BybitRead,BybitWrite};
use dotenvy::dotenv;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
fn sign_bybit(secret: &str, prehash: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(result)
}


pub async fn ceshi() -> anyhow::Result<()> {
    let bybit_qty = 140.to_string();
    let (mut b_write,b_read) = bybit_login().await?;
    bybit_open(&mut b_write, "FLOWUSDT", "Sell", &bybit_qty).await?;
    Ok(())
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
