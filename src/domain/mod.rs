//! Module for the (crate-internal) types defining the transaction domain.

use rust_decimal::Decimal;

pub(crate) type Money = Decimal;

/// Transactions are the orders provided to the engine.
pub(crate) enum Transaction {
    Deposit {
        client_id: ClientId,
        tx_id: TxId,
        amount: Money,
    },
}

/// Id identifying the client issuing the transaction.
#[derive(Debug)]
pub(crate) struct ClientId(u16);

/// The unique ID of a transaction. Used to reference transactions for disputes, resolves, and chargebacks
#[derive(Debug)]
pub(crate) struct TxId(u32);

/// The account state of a client
pub(crate) struct AccountState {
    available: Money,
    held: Money,
    locked: bool,
}
