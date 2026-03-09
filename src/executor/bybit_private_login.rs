use anyhow::Result;
use chrono::Utc;
use dotenvy::dotenv;
use futures_util::{SinkExt, StreamExt, stream::SplitStream};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::env;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream, connect_async,
    tungstenite::Message,
};

type HmacSha256 = Hmac<Sha256>;
pub type BybitWs = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type BybitRead = SplitStream<BybitWs>;

fn sign(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn bybit_private_login() -> Result<BybitRead> {
    dotenv().ok();

    let api_key = env::var("BYBIT_API_KEY")?;
    let secret = env::var("BYBIT_SECRET")?;

    let url = "wss://stream.bybit.com/v5/private";
    let (ws, _) = connect_async(url).await?;
    let (mut write, mut read) = ws.split();

    let expires = Utc::now().timestamp_millis() + 5000;
    let sign_payload = format!("GET/realtime{}", expires);
    let signature = sign(&secret, &sign_payload);

    let auth_msg = serde_json::json!({
        "op": "auth",
        "args": [api_key, expires.to_string(), signature]
    });

    write.send(Message::Text(auth_msg.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["op"] == "auth" {
                if v["success"].as_bool() == Some(true) {
                    break;
                } else {
                    anyhow::bail!("bybit auth failed: {}", text);
                }
            }
        }
    }

    let sub_msg = serde_json::json!({
        "op": "subscribe",
        "args": ["wallet"]
    });

    write.send(Message::Text(sub_msg.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["op"] == "subscribe" {
                if v["success"].as_bool() == Some(true) {
                    break;
                } else {
                    anyhow::bail!("bybit subscribe failed: {}", text);
                }
            }
        }
    }

    Ok(read)
}