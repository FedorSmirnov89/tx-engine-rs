use super::*;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[test]
fn single_account_converts_correctly() {
    let client_id = 1u16;
    let available = dec!(10.0);
    let held = dec!(5.0);
    let total = available + held;
    let locked = false;

    let mut accounts = HashMap::new();
    accounts.insert(
        ClientId::new(client_id),
        AccountState::new(available, held, locked),
    );

    let records: Vec<_> = to_account_records(accounts).collect();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0],
        AccountRecord {
            client: client_id,
            available,
            held,
            total,
            locked,
        }
    );
}

#[test]
fn total_is_sum_of_available_and_held() {
    let available = dec!(3.3333);
    let held = dec!(1.6667);
    let expected_total = available + held;

    let mut accounts = HashMap::new();
    accounts.insert(ClientId::new(1), AccountState::new(available, held, false));

    let record = to_account_records(accounts).next().unwrap();
    assert_eq!(record.total, expected_total);
}

#[test]
fn locked_account_preserves_locked_flag() {
    let locked = true;

    let mut accounts = HashMap::new();
    accounts.insert(
        ClientId::new(1),
        AccountState::new(dec!(0.0), dec!(0.0), locked),
    );

    let record = to_account_records(accounts).next().unwrap();
    assert_eq!(record.locked, locked);
}

#[test]
fn empty_map_yields_no_records() {
    let accounts = HashMap::new();
    let records: Vec<_> = to_account_records(accounts).collect();
    assert!(records.is_empty());
}

#[test]
fn multiple_accounts_yield_one_record_each() {
    let mut accounts = HashMap::new();
    accounts.insert(
        ClientId::new(1),
        AccountState::new(dec!(1.0), dec!(0.0), false),
    );
    accounts.insert(
        ClientId::new(2),
        AccountState::new(dec!(2.0), dec!(0.0), false),
    );

    let records: Vec<_> = to_account_records(accounts).collect();
    assert_eq!(records.len(), 2);
}
