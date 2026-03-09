use anyhow::{bail, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    MaybeTlsStream, WebSocketStream,
    tungstenite::Message,
};

pub type OkxWrite = futures_util::stream::SplitSink<
    WebSocketStream<MaybeTlsStream<TcpStream>>,
    Message,
>;

pub type OkxRead = futures_util::stream::SplitStream<
    WebSocketStream<MaybeTlsStream<TcpStream>>,
>;

pub async fn okx_open(
    write: &mut OkxWrite,
    inst_id: &str,
    side: &str,   // "buy" / "sell"
    sz: &str,
) -> Result<()> {
    if side != "buy" && side != "sell" {
        bail!("side must be buy or sell");
    }

    let order_msg = serde_json::json!({
        "id": "1",
        "op": "order",
        "args": [{
            "instId": inst_id,
            "tdMode": "cross",
            "side": side,
            "ordType": "market",
            "sz": sz
        }]
    });

    write
        .send(Message::Text(order_msg.to_string().into()))
        .await?;

    println!("OKX 已发送开仓单: side={}, sz={}", side, sz);
    Ok(())
}