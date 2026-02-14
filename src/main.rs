use anyhow::Result;
use tx_engine_rs::setup_logging;

fn main() -> Result<()> {
    setup_logging()?;

    tracing::info!("starting the tx-engine");

    println!("Hello, world!");

    Ok(())
}
