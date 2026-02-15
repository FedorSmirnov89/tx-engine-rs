mod domain;
mod engine;
mod error;
mod input;
mod output;
mod telemetry;

pub use error::Error;
pub use output::{AccountRecord, TransactionRecord};
pub use telemetry::setup_logging;

use crate::input::parse_transactions;

/// Processes financial transactions from a CSV source and returns per-client account records.
///
/// This is the single public entry point of the crate. It reads CSV-encoded transactions
/// from `reader`, applies them to in-memory account state, and yields the final balances
/// as an iterator of [`AccountRecord`]s ready for serialization.
///
/// # Callbacks
///
/// The caller controls what happens on success and failure through two callbacks:
///
/// - **`on_error`** — invoked for every transaction that cannot be processed (malformed CSV
///   row, domain validation failure, or a processing error such as insufficient funds).
///   The transaction is skipped and processing continues.
/// - **`on_success`** — invoked with each [`TransactionRecord`] that was
///   successfully applied. Useful for logging, metrics, publishing, or progress tracking.
///
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// use tx_engine_rs::{process, Error, TransactionRecord};
/// use tracing::{info, warn};
///
/// let reader = File::open("transactions.csv").unwrap();
/// let writer = std::io::stdout();
///
/// let mut wtr = csv::Writer::from_writer(writer);
/// for record in process(
///     reader,
///     |e: Error| warn!("skipped: {e}"),
///     |tx: TransactionRecord| info!("processed: {tx}"),
/// ) {
///     wtr.serialize(&record).unwrap();
/// }
/// wtr.flush().unwrap();
/// ```
#[must_use = "this iterator is lazy and must be consumed to process the account states"]
pub fn process(
    reader: impl std::io::Read,
    on_error: impl FnMut(Error),
    on_success: impl FnMut(TransactionRecord),
) -> impl Iterator<Item = AccountRecord> {
    let results = parse_transactions(reader);
    let accounts = engine::process_transactions(results, on_error, on_success);
    output::to_account_records(accounts)
}

/// Parallel variant — client-sharded, multi-threaded processing.
///
/// Designed for standalone batch processing of large inputs where
/// the engine itself must shard and parallelise. For pre-sharded streams
/// (e.g., in a distributed deployment), prefer [`process()`] which avoids
/// threading overhead entirely.
pub fn process_parallel(
    reader: impl std::io::Read,
    on_error: impl FnMut(Error) + Send,
    on_success: impl FnMut(TransactionRecord) + Send,
    num_workers: usize,
    channel_capacity: usize,
) -> impl Iterator<Item = AccountRecord> {
    let num_workers = if num_workers == 0 {
        tracing::warn!("num_workers set to 0, defaulting to 1");
        1
    } else {
        num_workers
    };

    let results = parse_transactions(reader);
    let accounts = engine::process_transactions_parallel(
        results,
        on_error,
        on_success,
        num_workers,
        channel_capacity,
    );
    output::to_account_records(accounts)
}
