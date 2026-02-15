//! Integration tests for withdrawal transactions

use rust_decimal_macros::dec;
use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

#[test]
fn deposit_then_withdraw() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, 4.0";

    let expected = AccountRecord {
        client: 1,
        available: dec!(6.0),
        held: dec!(0),
        total: dec!(6.0),
        locked: false,
    };

    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| panic!("unexpected error: {e}"),
        |tx| successful_txs.push(tx),
    )
    .collect();

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
        TransactionRecord::Withdrawal {
            client: 1,
            tx: 2,
            amount: dec!(4.0),
        }
    );
}

#[test]
fn deposit_then_two_withdrawals() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 20.0
withdrawal, 1, 2, 7.0
withdrawal, 1, 3, 3.0";

    let expected = AccountRecord {
        client: 1,
        available: dec!(10.0),
        held: dec!(0),
        total: dec!(10.0),
        locked: false,
    };

    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| panic!("unexpected error: {e}"),
        |tx| successful_txs.push(tx),
    )
    .collect();

    assert_eq!(records.len(), 1);
    assert_eq!(records[0], expected);
    assert_eq!(successful_txs.len(), 3);
}

#[test]
fn overdraft_then_deposit_then_valid_withdrawal() {
    let input = "\
type, client, tx, amount
withdrawal, 1, 1, 5.0
deposit, 1, 2, 10.0
withdrawal, 1, 3, 3.0";

    let expected = AccountRecord {
        client: 1,
        available: dec!(7.0),
        held: dec!(0),
        total: dec!(7.0),
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

    // The first withdrawal (tx 1) should fail — no funds yet
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
        "expected overdraft error for tx 1"
    );

    // The deposit (tx 2) and second withdrawal (tx 3) should succeed
    assert_eq!(successful_txs.len(), 2);
    assert_eq!(
        successful_txs[0],
        TransactionRecord::Deposit {
            client: 1,
            tx: 2,
            amount: dec!(10.0),
        }
    );
    assert_eq!(
        successful_txs[1],
        TransactionRecord::Withdrawal {
            client: 1,
            tx: 3,
            amount: dec!(3.0),
        }
    );
}

#[test]
fn withdrawal_with_zero_amount_is_rejected() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, 0.0";

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    // Only the deposit succeeds; the zero withdrawal is rejected
    assert_eq!(successful_txs.len(), 1);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].available, dec!(10.0));

    assert_eq!(errors.len(), 1);
    match &errors[0] {
        Error::Validation {
            client_id, tx_id, ..
        } => {
            assert_eq!(*client_id, 1);
            assert_eq!(*tx_id, 2);
        }
        _ => panic!("Expected a Validation error"),
    }
}

#[test]
fn withdrawal_with_negative_amount_is_rejected() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
withdrawal, 1, 2, -5.0";

    let mut errors: Vec<Error> = Vec::new();
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |tx| successful_txs.push(tx),
    )
    .collect();

    // Only the deposit succeeds; the negative withdrawal is rejected
    assert_eq!(successful_txs.len(), 1);
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].available, dec!(10.0));

    assert_eq!(errors.len(), 1);
    match &errors[0] {
        Error::Validation {
            client_id, tx_id, ..
        } => {
            assert_eq!(*client_id, 1);
            assert_eq!(*tx_id, 2);
        }
        _ => panic!("Expected a Validation error"),
    }
}
