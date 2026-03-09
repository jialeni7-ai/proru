mod coreFn;
mod executor;
mod model;
mod monitor;
mod ws_task;
use executor::{
    bybit_close::bybit_close, bybit_open::bybit_open, okx_close::okx_close, okx_open::okx_open,
};
use model::quote::{Exchange, Quote};
use monitor::{bybit_monitor::bybit_monitor, okx_monitor::okx_monitor};
use tokio::time::{Duration, sleep};

use anyhow::Result;
use chrono::Utc;
use dotenvy::dotenv;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};

type HmacSha256 = Hmac<Sha256>;

fn sign(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let api_key = env::var("BYBIT_API_KEY")?;
    let secret = env::var("BYBIT_SECRET")?;

    let url = "wss://stream.bybit.com/v5/private";
    let (ws, _) = connect_async(url).await?;
    println!("Bybit private 已连接");

    let (mut write, mut read) = ws.split();

    // 1. auth
    let expires = Utc::now().timestamp_millis() + 5000;
    let sign_payload = format!("GET/realtime{}", expires);
    let signature = sign(&secret, &sign_payload);

    let auth_msg = serde_json::json!({
        "op": "auth",
        "args": [
            api_key,
            expires.to_string(),
            signature
        ]
    });

    write
        .send(Message::Text(auth_msg.to_string().into()))
        .await?;
    println!("已发送 auth");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("auth返回: {}", text);

            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["op"] == "auth" {
                if v["success"].as_bool() == Some(true) {
                    println!("auth 成功");
                    break;
                } else {
                    anyhow::bail!("auth 失败: {}", text);
                }
            }
        }
    }

    // 2. subscribe wallet
    let sub_msg = serde_json::json!({
        "op": "subscribe",
        "args": ["wallet"]
    });

    write
        .send(Message::Text(sub_msg.to_string().into()))
        .await?;
    println!("已发送 wallet 订阅");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("收到消息: {}", text);

            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["op"] == "subscribe" {
                if v["success"].as_bool() == Some(true) {
                    println!("wallet 订阅成功");
                    break;
                } else {
                    anyhow::bail!("wallet 订阅失败: {}", text);
                }
            }
        }
    }

    // 3. 持续监听 wallet
    println!("开始监听 wallet...");

    while let Some(msg) = read.next().await {
        let msg = msg?;

        match msg {
            Message::Text(text) => {
                println!("原始消息: {}", text);

                let v: serde_json::Value = match serde_json::from_str(&text) {
                    Ok(v) => v,
                    Err(_) => continue,
                };

                if v["topic"] != "wallet" {
                    continue;
                }

                println!("wallet推送: {}", v);

                // 这里先别急着硬取字段，先看真实结构
            }
            other => {
                println!("非文本消息: {:?}", other);
            }
        }
    }

    anyhow::bail!("连接断开")
}
