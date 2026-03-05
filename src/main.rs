mod model;
mod ws_task;
mod coreFn;
mod executor;
mod monitor;
use model::quote::{Exchange,Quote};
use tokio::time::{sleep, Duration};
use executor::{okx_open::okx_open,okx_close::okx_close};
use monitor::okx_monitor::okx_monitor;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // okx_open().await?;
    okx_monitor().await?;
    sleep(Duration::from_secs(5)).await;
    // okx_close().await?;
    Ok(())
}


