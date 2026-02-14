//! Module for the core logic of the engine

use std::collections::HashMap;

use crate::domain::{AccountState, ClientId, Transaction};

///
/// Processes an iterator of transactions and outputs the final state of client accounts, once the iterator is empty.
///
pub(crate) fn process_transactions(
    transactions: impl IntoIterator<Item = Transaction>,
) -> HashMap<ClientId, AccountState> {
    todo!()
}
