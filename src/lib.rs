mod domain;
mod engine;
mod error;
mod input;
mod output;
mod telemetry;

pub use error::Error;
pub use output::AccountRecord;
pub use telemetry::setup_logging;

/// Processes financial transactions from a CSV source and returns per-client account records.
///
/// This is the single public entry point of the crate. It reads CSV-encoded transactions
/// from `reader`, applies them to in-memory account state, and yields the final balances
/// as an iterator of [`AccountRecord`]s ready for serialization.
///
/// # Error handling
///
/// Not every row in the input may be valid — the CSV can contain malformed rows or
/// transactions that violate domain rules (e.g. a deposit with a negative amount).
/// Instead of aborting on the first bad row, `process` reports each error to the
/// caller-supplied `on_error` callback and continues with the remaining input.
///
/// Please use the callback function to define the error hanling most appropriate for your use case.
///
///
/// # Example
///
/// ```no_run
/// use std::fs::File;
/// use tx_engine_rs::{process, Error};
///
/// let reader = File::open("transactions.csv").unwrap();
/// let writer = std::io::stdout();
///
/// let mut wtr = csv::Writer::from_writer(writer);
/// for record in process(reader, |e: Error| eprintln!("skipped: {e}")) {
///     wtr.serialize(&record).unwrap();
/// }
/// wtr.flush().unwrap();
/// ```
pub fn process(
    reader: impl std::io::Read,
    mut on_error: impl FnMut(Error),
) -> impl Iterator<Item = AccountRecord> {
    let transactions = input::parse_transactions(reader).filter_map(move |result| match result {
        Ok(tx) => Some(tx),
        Err(e) => {
            on_error(e);
            None
        }
    });
    let accounts = engine::process_transactions(transactions);
    output::to_account_records(accounts)
}
