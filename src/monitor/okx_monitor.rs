use anyhow::{Result, anyhow};
use base64::{Engine, engine::general_purpose};
use chrono::Utc;
use dotenvy::dotenv;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};

fn sign(secret: &str, prehash: &str) -> String {
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

pub async fn okx_monitor() -> Result<()> {
    dotenv().ok();
    let api_key = env::var("OKX_API_KEY")?;
    let secret = env::var("OKX_SECRET_KEY")?;
    let passphrase = env::var("OKX_PASSPHRASE")?;

    let url = "wss://ws.okx.com:8443/ws/v5/private";
    let (mut ws, _) = connect_async(url).await?;
    println!("已连接");

    let (mut write, mut read) = ws.split();
    let ts = Utc::now().timestamp().to_string();
    let prehash = format!("{ts}GET/users/self/verify");
    let sign = sign(&secret, &prehash);
    let login = serde_json::json!({
        "op":"login",
        "args":[{
            "apiKey":api_key,
            "passphrase":passphrase,
            "timestamp":ts,
            "sign":sign
        }]
    });

    write.send(Message::Text(login.to_string().into())).await?;
    println!("已发送login");

    let mut login_ok = false;
    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            if text.contains("\"code\":\"0\"") {
                break;
            }
        }
        break;
    }

    let account = serde_json::json!({
        "op":"subscribe",
        "args":[{
        "channel":"account"
        }]
    });
    write
        .send(Message::Text(account.to_string().into()))
        .await?;
    println!("已经发送检测");
    while let Some(msg) = read.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            if text.contains("mgnRatio") {
                let v: serde_json::Value = serde_json::from_str(&text)?;

                let ratio_str = v["data"][0]["details"][0]["mgnRatio"]
                    .as_str()
                    .unwrap_or("");

                if !ratio_str.is_empty() {
                    let ratio: f64 = ratio_str.parse()?;
                    println!("保证金率: {}", ratio);
                }
            }
        }
    }
    Ok(())
}
