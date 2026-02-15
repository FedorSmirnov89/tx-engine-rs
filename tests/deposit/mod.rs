//! Integration tests for deposit transactions

use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

use rust_decimal_macros::dec;

#[test]
fn single_deposit() {
    // Arrange
    let input = "\
type, client, tx, amount
deposit, 1, 1, 1.5";

    let expected = AccountRecord {
        client: 1,
        available: dec!(1.5),
        held: dec!(0),
        total: dec!(1.5),
        locked: false,
    };

    // Act
    let mut successful_txs: Vec<TransactionRecord> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| panic!("unexpected error: {e}"),
        |tx: TransactionRecord| successful_txs.push(tx),
    )
    .collect();

    // Assert
    assert_eq!(records.len(), 1);
    let actual = records.first().unwrap();
    assert_eq!(expected, *actual);

    assert_eq!(successful_txs.len(), 1);
    assert_eq!(
        successful_txs[0],
        TransactionRecord::Deposit {
            client: 1,
            tx: 1,
            amount: dec!(1.5),
        }
    );
}

#[test]
fn deposit_with_zero_amount_is_rejected() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 0";

    let mut errors: Vec<Error> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |_| panic!("unexpected success"),
    )
    .collect();

    assert!(records.is_empty(), "no account should be created");
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Validation {
                client_id: 1,
                tx_id: 1,
                ..
            }
        ),
        "expected a validation error for client 1, tx 1"
    );
}

#[test]
fn deposit_with_negative_amount_is_rejected() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, -5.0";

    let mut errors: Vec<Error> = Vec::new();
    let records: Vec<AccountRecord> = process(
        input.as_bytes(),
        |e| errors.push(e),
        |_| panic!("unexpected success"),
    )
    .collect();

    assert!(records.is_empty(), "no account should be created");
    assert_eq!(errors.len(), 1);
    assert!(
        matches!(
            &errors[0],
            Error::Validation {
                client_id: 1,
                tx_id: 1,
                ..
            }
        ),
        "expected a validation error for client 1, tx 1"
    );
}
