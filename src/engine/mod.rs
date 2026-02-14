//! Module for the core logic of the engine

use std::collections::HashMap;

use crate::{
    Error,
    domain::{AccountState, ClientId, Deposit, Transaction},
    output::TransactionRecord,
};

type Accounts = HashMap<ClientId, AccountState>;

///
/// Processes an iterator of transactions and outputs the final state of client accounts, once the iterator is empty.
///
pub(crate) fn process_transactions(
    transactions: impl IntoIterator<Item = Result<Transaction, Error>>,
    mut on_error: impl FnMut(Error),
    mut on_success: impl FnMut(TransactionRecord),
) -> HashMap<ClientId, AccountState> {
    let mut accounts = Accounts::default();

    for result in transactions {
        let tx = match result {
            Ok(tx) => tx,
            Err(err) => {
                on_error(err);
                continue;
            }
        };

        match handle_transaction(&tx, &mut accounts) {
            Ok(()) => on_success(TransactionRecord::from_domain(&tx)),
            Err(err) => on_error(err),
        }
    }

    accounts
}

fn handle_transaction(tx: &Transaction, accounts: &mut Accounts) -> Result<(), Error> {
    match tx {
        Transaction::Deposit(deposit) => {
            handle_deposit(deposit, accounts);
            Ok(())
        }
    }
}

fn handle_deposit(deposit: &Deposit, accounts: &mut Accounts) {
    let account = accounts.entry(deposit.client_id()).or_default();
    account.deposit(deposit.amount());
}
