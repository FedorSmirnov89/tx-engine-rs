use std::collections::HashMap;
use std::fmt;

use serde::Serialize;

use crate::domain::{AccountState, ClientId, Money, Transaction};

#[cfg(test)]
mod tests;

pub(crate) fn to_account_records(
    accounts: HashMap<ClientId, AccountState>,
) -> impl Iterator<Item = AccountRecord> {
    accounts
        .into_iter()
        .map(|(id, state)| AccountRecord::new(id, state))
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct AccountRecord {
    pub client: u16,
    pub available: Money,
    pub held: Money,
    pub total: Money,
    pub locked: bool,
}

impl AccountRecord {
    fn new(client_id: ClientId, account_state: AccountState) -> Self {
        let total = account_state.available_funds() + account_state.held_funds();
        Self {
            client: client_id.into(),
            available: account_state.available_funds(),
            held: account_state.held_funds(),
            total,
            locked: account_state.is_locked(),
        }
    }
}

/// Public DTO representing a successfully processed transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionRecord {
    Deposit { client: u16, tx: u32, amount: Money },
}

impl TransactionRecord {
    pub(crate) fn from_domain(tx: &Transaction) -> Self {
        match tx {
            Transaction::Deposit(d) => TransactionRecord::Deposit {
                client: d.client_id().into(),
                tx: d.tx_id().into(),
                amount: d.amount(),
            },
        }
    }
}

impl fmt::Display for TransactionRecord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionRecord::Deposit { client, tx, amount } => {
                write!(
                    f,
                    "Deposit {{ client: {client}, tx: {tx}, amount: {amount} }}"
                )
            }
        }
    }
}
