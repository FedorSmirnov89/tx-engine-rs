use crate::domain::{ClientId, Deposit, Transaction, TxId};
use claims::{assert_err, assert_matches, assert_ok};

use rstest::rstest;

use super::*;

/// Helper: parse a CSV string and collect all results.
fn parse_csv(input: &str) -> Vec<Result<Transaction>> {
    parse_transactions(input.as_bytes()).collect()
}

/// Helper: parse a CSV string, assert all rows succeed, return the transactions.
fn parse_csv_ok(input: &str) -> Vec<Transaction> {
    parse_csv(input)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
        .expect("all rows should parse successfully")
}

#[test]
fn two_deposits() {
    let input = "\
type, client, tx, amount
deposit, 1, 1, 1.0
deposit, 2, 2, 2.0";

    let txs = parse_csv_ok(input);
    assert_eq!(txs.len(), 2);

    assert_eq!(
        txs[0],
        Transaction::Deposit(
            Deposit::new(ClientId::new(1), TxId::new(1), "1.0".parse().unwrap()).unwrap()
        )
    );
    assert_eq!(
        txs[1],
        Transaction::Deposit(
            Deposit::new(ClientId::new(2), TxId::new(2), "2.0".parse().unwrap()).unwrap()
        )
    );
}

#[test]
fn empty_file_yields_no_transactions() {
    let input = "type, client, tx, amount";

    let txs = parse_csv_ok(input);
    assert!(txs.is_empty());
}

#[rstest]
fn parse_deposit(
    // client ID and tx ID not varied since all combinations are valid
    #[values("deposit", "invalid")] tx_type: &str,
    #[values("1.0", "0.0", "999999.9999", "-1.0", "-0.0001", "")] amount: &str,
) {
    // Arrange
    let client_id = 1u16;
    let tx_id = 1u32;

    let input = format!("type, client, tx, amount\n{tx_type}, {client_id}, {tx_id}, {amount}");
    let valid_types = ["deposit"];
    let is_valid = !amount.is_empty()
        && amount.parse::<Decimal>().unwrap() > Decimal::ZERO
        && valid_types.contains(&tx_type);

    // Act
    let results = parse_csv(&input);
    assert_eq!(results.len(), 1);

    if is_valid {
        let tx = assert_ok!(results.into_iter().next().unwrap());
        let amount = amount.parse::<Decimal>().unwrap();

        match tx_type {
            "deposit" => {
                assert_matches!(tx, Transaction::Deposit(d) if d == Deposit::new(ClientId::new(client_id), TxId::new(tx_id), amount).unwrap() )
            }
            _ => unreachable!("invalid type"),
        }
    } else {
        assert_err!(&results[0]);
    }
}
