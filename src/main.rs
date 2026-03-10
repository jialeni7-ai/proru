mod coreFn;
mod executor;
mod model;
mod monitor;
mod ws_task;

use anyhow::Result;
use crate::coreFn::engine::engine;
// use crate::coreFn::ceshi::ceshi;
#[tokio::main]
async fn main() -> Result<()> {
    engine().await?;
    // ceshi().await?;
    Ok(())
}