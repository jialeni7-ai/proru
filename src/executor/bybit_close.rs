use anyhow::Result;
use chrono::Utc;
use dotenvy::dotenv;
use futures_util::{SinkExt, StreamExt};
use hmac::{Hmac, Mac};
use serde_json::{self, to_string};
use sha2::Sha256;
use std::env;
use tokio_tungstenite::{connect_async, tungstenite::Message};

type HmacSha256 = Hmac<Sha256>;
fn sign(secret: &str, payload: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(payload.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn bybit_close() -> Result<()> {
    dotenv().ok();
    let api_key = env::var("BYBIT_API_KEY")?;
    let secret = env::var("BYBIT_SECRET")?;
    let url = "wss://stream.bybit.com/v5/trade";
    let (mut ws, _) = connect_async(url).await?;
    println!("已经连接");

    let (mut write, mut read) = ws.split();

    let expires = Utc::now().timestamp_millis() + 5000;
    let ts = Utc::now().timestamp_millis();
    let sign_payload = format!("GET/realtime{}", expires);

    let signature = sign(&secret, &sign_payload);

    let login_msg = serde_json::json!({
        "op":"auth",
        "args":[
            api_key,
            expires,
            signature
        ]
    });

    write
        .send(Message::Text(login_msg.to_string().into()))
        .await?;

    println!("已发送 login");

    while let Some(msg) = read.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            println!("login返回值：{}", text);
            break;
        }
    }

    let order_msg = serde_json::json!({
        "op":"order.create",
        "header":{
            "X-BAPI-TIMESTAMP":ts.to_string()
        },
        "args":[{

            "symbol":"IPUSDT",

            "side":"Sell",

            "orderType":"Market",

            "qty":"6",

            "category":"linear",

            "timeInForce":"IOC",

            "reduceOnly": true
        }]
    });
    write
        .send(Message::Text(order_msg.to_string().into()))
        .await?;
    println!("已经发送 BYBIT 平多");

    while let Some(msg) = read.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            println!("order返回: {}", text);

            break;
        }
    }

    Ok(())
}
