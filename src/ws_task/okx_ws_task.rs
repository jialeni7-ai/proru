use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio::time::{sleep, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message};


pub async fn okx_bbo_tbt_loop(inst_id:&str) -> Result<()> {
    let url = "wss://ws.okx.com:8443/ws/v5/public";

    loop {
        println!("okx连接中...");
        match connect_async(url).await {
            Ok((mut ws_stream,_)) => {
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
                            println!("{}",txt);
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
            Err(e)=> {
                println!("okx报错：{}",e);
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
}