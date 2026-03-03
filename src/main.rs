mod ws_task;
mod model;
use ws_task::okx_ws_task::{okx_bbo_tbt_loop};
use tokio::sync::watch;
use tokio::select;
use crate::model::{Quote, Exchange};
use ws_task::bybit_ws_task::{bybit_ws_task};
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (okx_tx,mut okx_rx) = watch::channel::<Option<Quote>>(None);
    tokio::spawn(async move{
        let _= okx_bbo_tbt_loop("SOL-USDT-SWAP",okx_tx).await;
    });

    let (_dummy_tx, mut bybit_rx) = watch::channel::<Option<Quote>>(None);
    let mut last_okx: Option<Quote> = None;
    let mut last_bybit: Option<Quote> = None;
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
    Ok(())
}








// use tokio::sync::watch;
// use tokio::time::{sleep,Duration};
// use tokio::select;


// #[tokio::main]
// async fn main() {
//     let (tx1,mut rx1) = watch::channel("okx".to_string());
//     let (tx2,mut rx2) = watch::channel("bybit".to_string());

//     tokio::spawn(async move{
//         let mut count = 1;
//         loop {
//             tx1.send(format!("okx-{}",count)).unwrap();
//             count += 1;
//             sleep(Duration::from_secs(1)).await;
//         }
//     });

//     tokio::spawn(async move {
//         let mut count = 1;
//         loop {
//             tx2.send(format!("bybit-{}",count)).unwrap();
//             count += 1;
//             sleep(Duration::from_secs(2)).await;
//         }
//     });

//     println!("开始监听");
//     loop {
//         select! {
//             _= rx1.changed() => {
//                 println!("收到okx数据：{}",*rx1.borrow());
//             }
//             _= rx2.changed() => {
//                 println!("收到bybit数据：{}",*rx2.borrow());
//             }
//         }
//     }

// }
