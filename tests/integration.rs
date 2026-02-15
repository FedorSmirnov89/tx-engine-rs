//! Integration tests for the transaction engine.

mod chargeback;
mod deposit;
mod dispute;
mod from_file;
mod resolve;
mod scenarios;
mod withdrawal;

use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

#[test]
fn empty_input_produces_no_output() {
    let input = "type, client, tx, amount\n";

    let mut successes: Vec<TransactionRecord> = Vec::new();
    let mut errors: Vec<Error> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successes.push(tx),
    )
    .collect();

    assert!(records.is_empty(), "expected no accounts");
    assert!(successes.is_empty(), "expected no successes");
    assert!(errors.is_empty(), "expected no errors");
}
