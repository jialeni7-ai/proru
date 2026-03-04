use crate::model::{Exchange, Quote};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize)]
struct BybitMsg {
    #[serde(default)]
    topic: String,
    #[serde(default)]
    r#type: String,
    ts: i64,
    cts: i64,
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

pub async fn bybit_ws_task(inst_id: &str, tx: watch::Sender<Option<Quote>>) -> anyhow::Result<()> {
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
                    } else {
                        println!("异常数据:{}", txt);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    break;
                }
            }
        }
        tx.send(None);
        sleep(Duration::from_secs(1)).await;
    }
}
