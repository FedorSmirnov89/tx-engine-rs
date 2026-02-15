//! "Manual" integration tests targeted mainly on the resolve mechanic

use rust_decimal_macros::dec;
use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

#[test]
fn deposit_dispute_then_resolve() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
resolve, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(10.0),
        held: dec!(0),
        total: dec!(10.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    assert_eq!(successful_txs.len(), 3);
    assert_eq!(
        successful_txs[0],
        TransactionRecord::Deposit {
            client: 1,
            tx: 1,
            amount: dec!(10.0),
        }
    );
    assert_eq!(
        successful_txs[1],
        TransactionRecord::Dispute { client: 1, tx: 1 }
    );
    assert_eq!(
        successful_txs[2],
        TransactionRecord::Resolve { client: 1, tx: 1 }
    );
}

#[test]
fn resolve_nonexistent_tx() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
resolve, 1, 3,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(10.0),
        held: dec!(0),
        total: dec!(10.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    assert_eq!(successful_txs.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Processing {
                client_id: 1,
                tx_id: 3,
                ..
            }
        ),
        "expected a processing error for resolving a nonexistent tx"
    );
}

#[test]
fn resolve_undisputed_deposit() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
resolve, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(10.0),
        held: dec!(0),
        total: dec!(10.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    // Only the deposit succeeds; the resolve on an undisputed tx errors
    assert_eq!(successful_txs.len(), 1);
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Processing {
                client_id: 1,
                tx_id: 1,
                ..
            }
        ),
        "expected a processing error for resolving an undisputed deposit"
    );
}

#[test]
fn double_resolve() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
resolve, 1, 1,
resolve, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(10.0),
        held: dec!(0),
        total: dec!(10.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    // Deposit, dispute, first resolve succeed; second resolve errors
    assert_eq!(successful_txs.len(), 3);
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Processing {
                client_id: 1,
                tx_id: 1,
                ..
            }
        ),
        "expected a processing error for the duplicate resolve"
    );
}

#[test]
fn resolve_then_re_dispute() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
resolve, 1, 1,
dispute, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(0),
        held: dec!(10.0),
        total: dec!(10.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    // All four transactions succeed: deposit, dispute, resolve, re-dispute
    assert_eq!(successful_txs.len(), 4);
}

#[test]
fn two_deposits_dispute_and_resolve_first() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 20.0
dispute, 1, 1,
resolve, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(30.0),
        held: dec!(0),
        total: dec!(30.0),
        locked: false,
    };

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    // Two deposits, dispute, resolve — all succeed
    assert_eq!(successful_txs.len(), 4);
}
