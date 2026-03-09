use anyhow::{bail, Result};
use futures_util::SinkExt;
use tokio_tungstenite::tungstenite::Message;

use crate::executor::bybit_open::BybitWrite;

pub async fn bybit_close(
    write: &mut BybitWrite,
    symbol: &str,
    side: &str,   // "Buy" 或 "Sell"
    qty: &str,
) -> Result<()> {
    if side != "Buy" && side != "Sell" {
        bail!("side must be Buy or Sell");
    }

    let ts = chrono::Utc::now().timestamp_millis();

    let order_msg = serde_json::json!({
        "op": "order.create",
        "header": {
            "X-BAPI-TIMESTAMP": ts.to_string()
        },
        "args": [{
            "symbol": symbol,
            "side": side,
            "orderType": "Market",
            "qty": qty,
            "category": "linear",
            "timeInForce": "IOC",
            "reduceOnly": true
        }]
    });

    write
        .send(Message::Text(order_msg.to_string().into()))
        .await?;

    println!(
        "Bybit 已发送平仓单: symbol={}, side={}, qty={}",
        symbol, side, qty
    );

    Ok(())
}