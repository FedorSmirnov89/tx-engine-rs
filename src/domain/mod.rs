//! Module for the (crate-internal) types defining the transaction domain.

use rust_decimal::Decimal;

pub(crate) type Money = Decimal;

/// Transactions are the orders provided to the engine.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Transaction {
    Deposit(Deposit),
}

/// Data of a deposit transaction
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Deposit {
    client_id: ClientId,
    tx_id: TxId,
    amount: Money,
}

impl Deposit {
    pub(crate) fn new(client_id: ClientId, tx_id: TxId, amount: Money) -> Result<Self, String> {
        if amount <= Decimal::ZERO {
            return Err("the deposited amount must be positive".to_string());
        }
        Ok(Self {
            client_id,
            tx_id,
            amount,
        })
    }
}

/// Id identifying the client issuing the transaction.
#[derive(Debug, PartialEq, Eq, Hash)]
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
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct TxId(u32);

impl TxId {
    pub(crate) fn new(id: u32) -> Self {
        Self(id)
    }
}

/// The account state of a client
pub(crate) struct AccountState {
    available: Money,
    held: Money,
    locked: bool,
}

impl AccountState {
    pub(crate) fn new(available: Money, held: Money, locked: bool) -> Self {
        Self {
            available,
            held,
            locked,
        }
    }

    pub(crate) fn available_funds(&self) -> Decimal {
        self.available
    }
    pub(crate) fn held_funds(&self) -> Decimal {
        self.held
    }
    pub(crate) fn is_locked(&self) -> bool {
        self.locked
    }
}
