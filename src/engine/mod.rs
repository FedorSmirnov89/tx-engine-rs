//! Module for the core logic of the engine

use std::collections::HashMap;

use crate::{
    Error,
    domain::{AccountState, ClientId, Deposit, Dispute, Transaction, Withdrawal},
    error::processing_error,
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
        Transaction::Withdrawal(withdrawal) => handle_withdrawal(withdrawal, accounts),
        Transaction::Dispute(dispute) => handle_dispute(dispute, accounts),
    }
}

fn handle_deposit(deposit: &Deposit, accounts: &mut Accounts) {
    let account = accounts.entry(deposit.client_id()).or_default();
    account.deposit(*deposit);
}

fn handle_withdrawal(withdrawal: &Withdrawal, accounts: &mut Accounts) -> Result<(), Error> {
    let client_id = withdrawal.client_id();
    let tx_id = withdrawal.tx_id();

    let Some(account) = accounts.get_mut(&client_id) else {
        return Err(processing_error(
            client_id,
            tx_id,
            "withdrawal from a client without account",
        ));
    };

    account
        .withdraw(withdrawal.amount())
        .map_err(|msg| processing_error(client_id, tx_id, msg))
}

fn handle_dispute(dispute: &Dispute, accounts: &mut Accounts) -> Result<(), Error> {
    let client_id = dispute.client_id();
    let disputed_tx = dispute.disputed_tx_id();

    let Some(account) = accounts.get_mut(&client_id) else {
        return Err(processing_error(
            client_id,
            disputed_tx,
            "dispute from a client without account",
        ));
    };

    account
        .dispute(disputed_tx)
        .map_err(|msg| processing_error(client_id, disputed_tx, msg))
}
