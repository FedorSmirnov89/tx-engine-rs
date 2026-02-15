//! "Manual" integration tests targeted mainly on the dispute mechanic

use rust_decimal_macros::dec;
use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

#[test]
fn deposit_then_dispute() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
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

    assert_eq!(successful_txs.len(), 2);
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
}

#[test]
fn dispute_nonexistent_tx() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 3,";

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

    // Only the deposit succeeds; the dispute on a nonexistent tx errors
    assert_eq!(successful_txs.len(), 1);
    assert_eq!(
        successful_txs[0],
        TransactionRecord::Deposit {
            client: 1,
            tx: 1,
            amount: dec!(10.0),
        }
    );

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
        "expected a processing error for the dispute on nonexistent tx 3"
    );
}

#[test]
fn dispute_a_withdrawal() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, 5.0
dispute, 1, 2,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(5.0),
        held: dec!(0),
        total: dec!(5.0),
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

    // Deposit and withdrawal succeed; dispute on the withdrawal errors
    assert_eq!(successful_txs.len(), 2);
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Processing {
                client_id: 1,
                tx_id: 2,
                ..
            }
        ),
        "expected a processing error for disputing a withdrawal"
    );
}

#[test]
fn dispute_insufficient_funds() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, 8.0
dispute, 1, 1,";

    // available = 2 after the withdrawal, but the dispute needs 10 → fails
    let expected = AccountRecord {
        client: 1,
        available: dec!(2.0),
        held: dec!(0),
        total: dec!(2.0),
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

    // Deposit and withdrawal succeed; the dispute fails
    assert_eq!(successful_txs.len(), 2);
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
        "expected a processing error for dispute with insufficient available funds"
    );
}

#[test]
fn double_dispute_same_tx() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
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

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);

    // Deposit and first dispute succeed; second dispute errors
    assert_eq!(successful_txs.len(), 2);
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
        "expected a processing error for the duplicate dispute"
    );
}

#[test]
fn two_deposits_dispute_first() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 5.0
dispute, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(5.0),
        held: dec!(10.0),
        total: dec!(15.0),
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

    // All three succeed: two deposits + one dispute
    assert_eq!(successful_txs.len(), 3);
}
