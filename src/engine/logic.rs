//! Module focused on the logic of processing individual transactions.

use crate::{
    Error,
    domain::{
        AccountState, Chargeback, ClientId, Deposit, Dispute, Resolve, Transaction, TxId,
        Withdrawal,
    },
    engine::Accounts,
    error::processing_error,
    input::{TYPE_KW_CHARGEBACK, TYPE_KW_DISPUTE, TYPE_KW_RESOLVE},
};

pub(super) fn handle_transaction(tx: &Transaction, accounts: &mut Accounts) -> Result<(), Error> {
    match tx {
        Transaction::Deposit(deposit) => handle_deposit(deposit, accounts),
        Transaction::Withdrawal(withdrawal) => handle_withdrawal(withdrawal, accounts),
        Transaction::Dispute(dispute) => handle_dispute(dispute, accounts),
        Transaction::Resolve(resolve) => handle_resolve(resolve, accounts),
        Transaction::Chargeback(chargeback) => handle_chargeback(chargeback, accounts),
    }
}

fn handle_deposit(deposit: &Deposit, accounts: &mut Accounts) -> Result<(), Error> {
    let client_id = deposit.client_id();
    let tx_id = deposit.tx_id();

    let account = accounts.entry(deposit.client_id()).or_default();
    account
        .deposit(*deposit)
        .map_err(|msg| processing_error(client_id, tx_id, msg))
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

    let account = ensure_client_is_known(client_id, disputed_tx, TYPE_KW_DISPUTE, accounts)?;
    account
        .dispute(disputed_tx)
        .map_err(|msg| processing_error(client_id, disputed_tx, msg))
}

fn handle_resolve(resolve: &Resolve, accounts: &mut Accounts) -> Result<(), Error> {
    let client_id = resolve.client_id();
    let resolved_tx = resolve.resolved_tx_id();

    let account = ensure_client_is_known(client_id, resolved_tx, TYPE_KW_RESOLVE, accounts)?;
    account
        .resolve(resolved_tx)
        .map_err(|msg| processing_error(client_id, resolved_tx, msg))
}

fn handle_chargeback(chargeback: &Chargeback, accounts: &mut Accounts) -> Result<(), Error> {
    let client_id = chargeback.client_id();
    let reverted_tx = chargeback.reverted_tx_id();

    let account = ensure_client_is_known(client_id, reverted_tx, TYPE_KW_CHARGEBACK, accounts)?;
    account
        .chargeback(reverted_tx)
        .map_err(|msg| processing_error(client_id, reverted_tx, msg))
}

fn ensure_client_is_known<'a>(
    client_id: ClientId,
    tx_id: TxId,
    tx_type: &'static str,
    accounts: &'a mut Accounts,
) -> Result<&'a mut AccountState, Error> {
    accounts.get_mut(&client_id).ok_or(processing_error(
        client_id,
        tx_id,
        format!("{tx_type} from a client without account"),
    ))
}
