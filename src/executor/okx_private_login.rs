use anyhow::Result;
use base64::{Engine, engine::general_purpose};
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
pub type OkxWs = WebSocketStream<MaybeTlsStream<TcpStream>>;
pub type OkxRead = SplitStream<OkxWs>;

fn sign(secret: &str, prehash: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(prehash.as_bytes());
    general_purpose::STANDARD.encode(mac.finalize().into_bytes())
}

pub async fn okx_private_login() -> Result<OkxRead> {
    dotenv().ok();

    let api_key = env::var("OKX_API_KEY")?;
    let secret = env::var("OKX_SECRET_KEY")?;
    let passphrase = env::var("OKX_PASSPHRASE")?;

    let url = "wss://ws.okx.com:8443/ws/v5/private";
    let (ws, _) = connect_async(url).await?;
    let (mut write, mut read) = ws.split();

    let ts = Utc::now().timestamp().to_string();
    let prehash = format!("{ts}GET/users/self/verify");
    let sign = sign(&secret, &prehash);

    let login_msg = serde_json::json!({
        "op": "login",
        "args": [{
            "apiKey": api_key,
            "passphrase": passphrase,
            "timestamp": ts,
            "sign": sign
        }]
    });

    write.send(Message::Text(login_msg.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["event"] == "login" || v["op"] == "login" {
                if v["code"].as_str() == Some("0") || v["success"].as_bool() == Some(true) {
                    break;
                } else {
                    anyhow::bail!("okx login failed: {}", text);
                }
            }
        }
    }

    let sub_msg = serde_json::json!({
        "op":"subscribe",
        "args":[{"channel":"account"}]
    });

    write.send(Message::Text(sub_msg.to_string().into())).await?;

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if v["event"] == "subscribe" && v["arg"]["channel"] == "account" {
                break;
            }
        }
    }

    Ok(read)
}