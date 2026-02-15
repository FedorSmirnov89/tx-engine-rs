//! "Manual" integration tests targeted mainly on the chargeback mechanic

use rust_decimal_macros::dec;
use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

#[test]
fn deposit_dispute_then_chargeback() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
chargeback, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(0),
        held: dec!(0),
        total: dec!(0),
        locked: true,
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
        TransactionRecord::Chargeback { client: 1, tx: 1 }
    );
}

#[test]
fn chargeback_nonexistent_tx() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
chargeback, 1, 3,";

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
        "expected a processing error for chargeback on nonexistent tx"
    );
}

#[test]
fn chargeback_undisputed_deposit() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
chargeback, 1, 1,";

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
                tx_id: 1,
                ..
            }
        ),
        "expected a processing error for chargeback on undisputed deposit"
    );
}

#[test]
fn double_chargeback() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
chargeback, 1, 1,
chargeback, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(0),
        held: dec!(0),
        total: dec!(0),
        locked: true,
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

    // Deposit, dispute, first chargeback succeed; second chargeback errors (account frozen)
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
        "expected a processing error for the duplicate chargeback"
    );
}

#[test]
fn frozen_account_rejects_deposit() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
dispute, 1, 1,
chargeback, 1, 1,
deposit, 1, 2, 5.0";

    let expected = AccountRecord {
        client: 1,
        available: dec!(0),
        held: dec!(0),
        total: dec!(0),
        locked: true,
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

    // Deposit, dispute, chargeback succeed; second deposit rejected (frozen)
    assert_eq!(successful_txs.len(), 3);
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
        "expected a processing error for deposit on frozen account"
    );
}

#[test]
fn frozen_account_rejects_withdrawal() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 5.0
dispute, 1, 1,
chargeback, 1, 1,
withdrawal, 1, 3, 3.0";

    let expected = AccountRecord {
        client: 1,
        available: dec!(5),
        held: dec!(0),
        total: dec!(5),
        locked: true,
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

    // Two deposits, dispute, chargeback succeed; withdrawal rejected (frozen)
    assert_eq!(successful_txs.len(), 4);
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
        "expected a processing error for withdrawal on frozen account"
    );
}

#[test]
fn frozen_account_rejects_dispute() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 5.0
dispute, 1, 1,
chargeback, 1, 1,
dispute, 1, 2,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(5),
        held: dec!(0),
        total: dec!(5),
        locked: true,
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

    // Two deposits, dispute, chargeback succeed; second dispute rejected (frozen)
    assert_eq!(successful_txs.len(), 4);
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
        "expected a processing error for dispute on frozen account"
    );
}

#[test]
fn two_deposits_dispute_and_chargeback_first() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 20.0
dispute, 1, 1,
chargeback, 1, 1,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(20),
        held: dec!(0),
        total: dec!(20),
        locked: true,
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

    // Two deposits, dispute, chargeback — all succeed
    assert_eq!(successful_txs.len(), 4);
}

#[test]
fn frozen_account_rejects_chargeback_on_other_dispute() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 10.0
deposit, 1, 2, 20.0
dispute, 1, 1,
dispute, 1, 2,
chargeback, 1, 1,
chargeback, 1, 2,";

    let expected = AccountRecord {
        client: 1,
        available: dec!(0),
        held: dec!(20),
        total: dec!(20),
        locked: true,
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

    // Two deposits, two disputes, first chargeback — all succeed;
    // second chargeback rejected (account frozen)
    assert_eq!(successful_txs.len(), 5);
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
        "expected a processing error for chargeback on frozen account"
    );
}
