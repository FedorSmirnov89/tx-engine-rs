//! Module for the types defining the transaction domain.

use rust_decimal::Decimal;

mod account;
mod transaction;

pub(crate) use account::AccountState;
pub(crate) use transaction::{Chargeback, Deposit, Dispute, Resolve, Transaction, Withdrawal};

pub(crate) type Money = Decimal;

/// Id identifying the client issuing the transaction.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) struct ClientId(u16);

impl ClientId {
    pub(crate) fn new(id: u16) -> Self {
        Self(id)
    }
}

impl From<ClientId> for u16 {
    fn from(value: ClientId) -> Self {
        value.0
    }
}

/// The unique ID of a transaction. Used to reference transactions for disputes, resolves, and chargebacks
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub(crate) struct TxId(u32);

impl TxId {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

impl From<TxId> for u32 {
    fn from(value: TxId) -> Self {
        value.0
    }
}
