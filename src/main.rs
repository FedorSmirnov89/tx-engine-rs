use anyhow::Result;
use std::{env, fs::File};
use tx_engine_rs::{Error, TransactionRecord, process, setup_logging};

fn main() -> Result<()> {
    setup_logging();

    let reader = get_reader()?;
    let writer = get_writer();

    let mut wtr = csv::Writer::from_writer(writer);
    for record in process(reader, handle_tx_error, handle_tx_success) {
        wtr.serialize(&record)?;
    }
    wtr.flush()?;

    Ok(())
}

fn get_reader() -> Result<impl std::io::Read> {
    let path = env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: tx-engine-rs <input.csv>"))?;
    let file = File::open(&path)?;
    Ok(file)
}

fn get_writer() -> impl std::io::Write {
    std::io::stdout()
}

fn handle_tx_error(error: Error) {
    tracing::warn!("{error}");
}

fn handle_tx_success(tx: TransactionRecord) {
    tracing::info!("Transaction accepted: {tx}");
}
