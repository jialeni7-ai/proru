use crate::model::quote::{Exchange,Quote};
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use tokio::sync::watch;
use tokio::time::{Duration, sleep};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Deserialize)]
struct OkxMsg {
    data: Vec<OkxData>,
}

#[derive(Debug, Deserialize)]
struct OkxData {
    asks: Vec<[String; 4]>,
    bids: Vec<[String; 4]>,
    ts: String,
    #[serde(rename = "seqId")]
    seq_id: u64,
}

fn parse_okx_bbo(txt: &str) -> Option<Quote> {
    let msg: OkxMsg = serde_json::from_str(txt).ok()?;
    let d = msg.data.get(0)?;

    let ask1 = d.asks.get(0)?;
    let bid1 = d.bids.get(0)?;

    Some(Quote {
        exchange: Exchange::Okx,
        bid_px: bid1[0].parse().ok()?,
        bid_sz: bid1[1].parse().ok()?,
        ask_px: ask1[0].parse().ok()?,
        ask_sz: ask1[1].parse().ok()?,
        ts: d.ts.parse().ok()?,
    })
}
pub async fn okx_bbo_tbt_loop(inst_id: &str, tx: watch::Sender<Option<Quote>>) -> Result<()> {
    let url = "wss://ws.okx.com:8443/ws/v5/public";

    loop {
        println!("okx连接中...");
        match connect_async(url).await {
            Ok((mut ws_stream, _)) => {
                println!("okx已连接");
                let subscribe_msg = json!({
                    "op": "subscribe",
                    "args": [{
                        "channel": "bbo-tbt",
                        "instId": inst_id
                    }]
                })
                .to_string();
                ws_stream.send(Message::Text(subscribe_msg.into())).await?;

                while let Some(msg) = ws_stream.next().await {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            if let Some(bbo) = parse_okx_bbo(&txt) {
                                let _ = tx.send(Some(bbo));
                            } else {
                                println!("其他数据：{}", txt);
                            }
                        }
                        Ok(Message::Ping(payload)) => {
                            ws_stream.send(Message::Pong(payload)).await?;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            println!("[okx] ws error: {e:?}");
                            break;
                        }
                    }
                }
            }
            Err(e) => {
                println!("okx报错：{}", e);
            }
        }
        let _ = tx.send(None);
        sleep(Duration::from_secs(1)).await;
    }
}
