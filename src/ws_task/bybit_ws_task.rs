use crate::model::quote::{Exchange,Quote};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize)]
struct BybitMsg {
    #[serde(default)]
    topic: String,
    #[serde(default)]
    r#type: String,
    ts: u64,
    cts: u64,
    #[serde(default)]
    data: BybitData,
}

#[derive(Debug, Deserialize, Default)]
struct BybitData {
    s: String,
    #[serde(default)]
    b: Vec<[String; 2]>,
    #[serde(default)]
    a: Vec<[String; 2]>,
}

fn parse_bybit_bbo(txt: &str) -> Option<Quote> {
    let msg: BybitMsg = serde_json::from_str(txt).ok()?;

    let bid1 = msg.data.b.get(0)?;
    let ask1 = msg.data.a.get(0)?;
    Some(Quote {
        exchange: Exchange::Bybit,
        bid_px: bid1[0].parse().ok()?,
        bid_sz: bid1[1].parse().ok()?,
        ask_px: ask1[0].parse().ok()?,
        ask_sz: ask1[1].parse().ok()?,
        ts: msg.cts,
    })
}

pub async fn bybit_ws_task(inst_id: &str, tx: watch::Sender<Option<Quote>>) -> Result<()> {
    let url = "wss://stream.bybit.com/v5/public/linear";
    let sub = serde_json::json! ({
        "op" : "subscribe",
        "args" : [format!("orderbook.1.{inst_id}")]
    }).to_string();
    loop {
        let (mut ws, _) = connect_async(url).await?;
        ws.send(Message::Text((sub.clone().into()))).await?;
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(txt)) => {
                    if let Some(bbo) = parse_bybit_bbo(&txt) {
                        let _ = tx.send(Some(bbo));
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    println!("bybit报错：{}",e);
                    break;
                }
            }
        }
        let _ = tx.send(None);
        sleep(Duration::from_secs(1)).await;
    }
}
