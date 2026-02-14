use std::collections::HashMap;

use serde::Serialize;

use crate::domain::{AccountState, ClientId, Money};

#[cfg(test)]
mod tests;

pub(crate) fn to_account_records(
    accounts: HashMap<ClientId, AccountState>,
) -> impl Iterator<Item = AccountRecord> {
    accounts
        .into_iter()
        .map(|(id, state)| AccountRecord::new(id, state))
}

#[derive(Serialize, Debug, PartialEq)]
pub struct AccountRecord {
    client: u16,
    available: Money,
    held: Money,
    total: Money,
    locked: bool,
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
