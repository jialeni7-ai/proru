mod model;
mod ws_task;
mod coreFn;

use model::quote::{Exchange,Quote};
use serde::Deserialize;
use tokio::select;
use tokio::sync::watch;


use ws_task::bybit_ws_task::bybit_ws_task;
use ws_task::okx_ws_task::okx_bbo_tbt_loop;
use ws_task::get_okx_ct_mult::get_okx_ct_mult;
use coreFn::engine;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    engine::engine().await?;
    Ok(())
}


