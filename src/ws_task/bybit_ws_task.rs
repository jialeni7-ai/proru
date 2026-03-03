use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio::sync::watch;
use crate::model::{Quote};

async fn bybit_ws_task(inst_id:&str,tx:watch::Sender<Quote>) -> anyhow::Result<()> {
    let url = "wss://stream.bybit.com/v5/public/linear";
    let (mut ws,_) = connect_async(url).await?;

    let sub= serde_json::json! ({
        "op" : "subscribe",
        "args" : [format!("orderbook.1.{inst_id}")]
    });

    while let Some(msg) = ws.next().await {
        let 
    }
    Ok(())
}