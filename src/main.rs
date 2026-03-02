mod ws_task;

#[tokio::main]
async fn main() ->anyhow::Result<()> {
    tokio::spawn(async {
        let _= ws_task::okx_ws_task::okx_bbo_tbt_loop("SOL-USDT-SWAP").await;
    });
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}