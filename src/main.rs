mod coreFn;
mod executor;
mod model;
mod monitor;
mod ws_task;

use anyhow::Result;
use crate::coreFn::engine::engine;

#[tokio::main]
async fn main() -> Result<()> {
    engine().await?;
    Ok(())
}