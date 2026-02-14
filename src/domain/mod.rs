//! Module for the types defining the transaction domain.

use rust_decimal::Decimal;

pub(crate) type Money = Decimal;

/// Transactions are the orders provided to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Withdrawal {
    client_id: ClientId,
    tx_id: TxId,
    amount: Money,
}

impl Withdrawal {
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

    pub(crate) fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub(crate) fn tx_id(&self) -> TxId {
        self.tx_id
    }

    pub(crate) fn amount(&self) -> Money {
        self.amount
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    pub(crate) fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub(crate) fn tx_id(&self) -> TxId {
        self.tx_id
    }

    pub(crate) fn amount(&self) -> Money {
        self.amount
    }
}

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
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
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

/// The account state of a client
#[derive(Debug, Default)]
pub(crate) struct AccountState {
    available: Money,
    held: Money,
    locked: bool,
}

impl AccountState {
    #[cfg(test)]
    pub(crate) fn new(available: Money, held: Money, locked: bool) -> Self {
        Self {
            available,
            held,
            locked,
        }
    }

    pub(crate) fn deposit(&mut self, amount: Money) {
        self.available += amount;
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
