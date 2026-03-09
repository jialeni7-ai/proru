use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;

use crate::executor::okx_open::OkxRead;

pub async fn okx_monitor(
    mut read: OkxRead,
    tx: watch::Sender<Option<f64>>,
) -> Result<()> {
    while let Some(msg) = read.next().await {
        let msg = msg?;

        if let Message::Text(text) = msg {
            let v: serde_json::Value = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let ratio_str = v["data"][0]["details"][0]["mgnRatio"]
                .as_str()
                .unwrap_or("");

            if ratio_str.is_empty() {
                continue;
            }

            let ratio: f64 = ratio_str.parse()?;
            let _ = tx.send(Some(ratio));

            println!("OKX 保证金率: {}", ratio);
        }
    }

    anyhow::bail!("okx monitor read closed")
}