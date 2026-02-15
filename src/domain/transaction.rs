//! Module defining the domain types related to the representation of the transactions handled by the engine

use rust_decimal::Decimal;

use crate::domain::{ClientId, Money, TxId};

/// Transactions are the orders provided to the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Transaction {
    Deposit(Deposit),
    Withdrawal(Withdrawal),
    Dispute(Dispute),
    Resolve(Resolve),
    Chargeback(Chargeback),
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
            return Err("the withdrawn amount must be positive".to_string());
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Dispute {
    client_id: ClientId,
    disputed_tx: TxId,
}

impl Dispute {
    pub(crate) fn new(client_id: ClientId, disputed_tx: TxId) -> Self {
        Self {
            client_id,
            disputed_tx,
        }
    }

    pub(crate) fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub(crate) fn disputed_tx_id(&self) -> TxId {
        self.disputed_tx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Resolve {
    client_id: ClientId,
    resolved_tx: TxId,
}

impl Resolve {
    pub(crate) fn new(client_id: ClientId, resolved_tx: TxId) -> Self {
        Self {
            client_id,
            resolved_tx,
        }
    }

    pub(crate) fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub(crate) fn resolved_tx_id(&self) -> TxId {
        self.resolved_tx
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Chargeback {
    client_id: ClientId,
    reverted_tx: TxId,
}

impl Chargeback {
    pub(crate) fn new(client_id: ClientId, reverted_tx: TxId) -> Self {
        Self {
            client_id,
            reverted_tx,
        }
    }

    pub(crate) fn client_id(&self) -> ClientId {
        self.client_id
    }

    pub(crate) fn reverted_tx_id(&self) -> TxId {
        self.reverted_tx
    }
}
