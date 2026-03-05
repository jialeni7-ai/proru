use crate::coreFn::strategy::strategy;
use crate::model::{quote::{Exchange, Quote},signal::{State,Signal}};
use crate::ws_task::{bybit_ws_task::bybit_ws_task, okx_ws_task::okx_bbo_tbt_loop};
use tokio::select;
use tokio::sync::watch;
use serde::Deserialize;


pub async fn engine() -> anyhow::Result<()> {
    let ct_mult = get_okx_ct_mult("SOL-USDT-SWAP").await?;
    let mut state = State::Idle;
    let (okx_tx, mut okx_rx) = watch::channel::<Option<Quote>>(None);
    let (bybit_tx,mut bybit_rx) = watch::channel::<Option<Quote>>(None);
    tokio::spawn(async move {
        let _ = okx_bbo_tbt_loop("SOL-USDT-SWAP", okx_tx).await;
    });
    tokio::spawn(async move {
        let _ = bybit_ws_task("SOLUSDT", bybit_tx).await;
    });

    let mut last_okx: Option<Quote> = None;
    let mut last_bybit: Option<Quote> = None;

    loop {
        select! {
            r = okx_rx.changed() => {
                if r.is_err() { break; }
                last_okx = okx_rx.borrow_and_update().clone();
            }
            r = bybit_rx.changed() => {
                if r.is_err() {break; }
                last_bybit = bybit_rx.borrow_and_update().clone();
            }
        }

        if let (Some(okx),Some(bybit)) = (&last_okx,&last_bybit) {
            if let Some(signal) = strategy(okx, bybit, ct_mult,&mut state) {
                match signal {
                    Signal::OpenOkxLongBybitShort => {
                        println!("开仓 OKX多 BYBIT空");
                    }
                    Signal::Close => {
                        println!("平仓");
                    }
                }
            }
        }
    }
    Ok(())
}




#[derive(Deserialize)]
struct OkxInstResp {
    data: Vec<OkxInstData>,
}
#[derive(Deserialize)]
struct OkxInstData {
    #[serde(rename="ctVal")]
    ct_val: String,
}

pub async fn get_okx_ct_mult(inst_id: &str) -> anyhow::Result<f64> {
    let url = format!(
        "https://www.okx.com/api/v5/public/instruments?instType=SWAP&instId={}",
        inst_id
    );
        let resp: OkxInstResp = reqwest::get(&url).await?.json().await?;

    let ct_mult = resp.data
        .get(0)
        .ok_or(anyhow::anyhow!("no instrument"))?
        .ct_val
        .parse::<f64>()?;

    Ok(ct_mult)
}
