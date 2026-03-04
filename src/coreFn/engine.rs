use crate::model::quote::{Exchange, Quote};
use crate::ws_task::{bybit_ws_task::bybit_ws_task, okx_ws_task::okx_bbo_tbt_loop};
use tokio::select;
use tokio::sync::watch;

pub async fn engine() -> anyhow::Result<()> {
    let (okx_tx, mut okx_rx) = watch::channel::<Option<Quote>>(None);
    tokio::spawn(async move {
        let _ = okx_bbo_tbt_loop("SOL-USDT-SWAP", okx_tx).await;
    });
    let mut last_okx: Option<Quote> = None;
    loop {
        select! {
            r = okx_rx.changed() => {
                if r.is_err() { break; }
                last_okx = okx_rx.borrow_and_update().clone();
                match &last_okx {
                    Some(bbo) => {
                        println!("[OKX] bid {}@{} ask {}@{} ts={}",
                            bbo.bid_sz, bbo.bid_px, bbo.ask_sz, bbo.ask_px, bbo.ts
                        );
                    }
                    None => println!("[OKX] 断线/暂无数据"),
                }
            }
        }
    }
    // let mut last_bybit: Option<Quote> = None;
    // let (tx_bybit, mut rx_bybit) = watch::channel::<Option<Quote>>(None);
    // tokio::spawn(async move {
    //     if let Err(e) = bybit_ws_task("BTCUSDT", tx_bybit).await {
    //         eprintln!("bybit task err:{}", e);
    //     }
    // });
    // loop {
    //     select! {
    //         r = rx_bybit.changed() => {
    //             if r.is_err() {break;}
    //             last_bybit = rx_bybit.borrow_and_update().clone();
    //             if let Some(q) = last_bybit.as_ref() {
    //                 println!("[BYBIT] {:?}", q);
    //             }
    //         }
    //     }
    // }

    Ok(())
}
