mod coreFn;
mod executor;
mod model;
mod monitor;
mod ws_task;
use executor::{
    bybit_close::bybit_close, bybit_open::bybit_open, okx_close::okx_close, okx_open::okx_open,
};
use model::quote::{Exchange, Quote};
use monitor::{bybit_monitor::bybit_monitor, okx_monitor::okx_monitor};
use tokio::time::{Duration, sleep};

use crate::executor::bybit_close;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // bybit_open().await?;
    bybit_monitor().await?;
    // sleep(Duration::from_secs(5)).await;
    // bybit_close().await?;
    Ok(())
}
