use anyhow::{bail, Result};
use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    tungstenite::Message,
    MaybeTlsStream, WebSocketStream,
};

pub type OkxWrite = futures_util::stream::SplitSink<
    WebSocketStream<MaybeTlsStream<TcpStream>>,
    Message,
>;

pub async fn okx_close(
    write: &mut OkxWrite,
    inst_id: &str,
    side: &str, // "buy" 或 "sell"
    sz: &str,
) -> Result<()> {
    if side != "buy" && side != "sell" {
        bail!("side must be buy or sell");
    }

    let order_msg = serde_json::json!({
        "id": "okx-close-1",
        "op": "order",
        "args": [{
            "instId": inst_id,
            "tdMode": "cross",
            "side": side,
            "ordType": "market",
            "sz": sz,
            "reduceOnly": true
        }]
    });

    write.send(Message::Text(order_msg.to_string().into())).await?;
    println!(
        "OKX 已发送平仓单: inst_id={}, side={}, sz={}",
        inst_id, side, sz
    );

    Ok(())
}