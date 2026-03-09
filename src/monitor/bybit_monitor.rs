use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;

use crate::executor::bybit_open::BybitRead;

pub async fn bybit_monitor(mut read: BybitRead, tx: watch::Sender<Option<f64>>) -> Result<()> {
    while let Some(msg) = read.next().await {
        let msg = msg?;

        let text = match msg {
            Message::Text(text) => text,
            _ => continue,
        };

        let v: serde_json::Value = match serde_json::from_str(&text) {
            Ok(v) => v,
            Err(_) => continue,
        };

        // 只处理 wallet 推送
        if v["topic"] != "wallet" {
            continue;
        }

        let Some(data_arr) = v["data"].as_array() else {
            continue;
        };

        let Some(first) = data_arr.first() else {
            continue;
        };

        let Some(mm_str) = first["accountMMRate"].as_str() else {
            continue;
        };

        let Ok(mm_rate) = mm_str.parse::<f64>() else {
            continue;
        };

        let _ = tx.send(Some(mm_rate));

        println!("Bybit accountMMRate: {}", mm_rate);
    }

    anyhow::bail!("bybit monitor read closed")
}
