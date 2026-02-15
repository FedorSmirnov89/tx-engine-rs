//! Module for the types defining the transaction domain.

use std::collections::HashMap;

use rust_decimal::Decimal;

pub(crate) type Money = Decimal;

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

/// The account state of a client
#[derive(Debug, Default)]
pub(crate) struct AccountState {
    accepted_deposits: HashMap<TxId, Money>,
    disputed_deposits: HashMap<TxId, Money>,

    available: Money,
    held: Money,
    locked: bool,
}

impl AccountState {
    #[cfg(test)]
    pub(crate) fn new(available: Money, held: Money, locked: bool) -> Self {
        Self {
            accepted_deposits: HashMap::new(),
            disputed_deposits: HashMap::new(),
            available,
            held,
            locked,
        }
    }

    pub(crate) fn deposit(&mut self, deposit: Deposit) -> Result<(), String> {
        self.ensure_not_locked()?;

        self.available += deposit.amount();
        self.accepted_deposits
            .insert(deposit.tx_id(), deposit.amount());
        Ok(())
    }

    pub(crate) fn withdraw(&mut self, amount: Money) -> Result<(), String> {
        self.ensure_not_locked()?;

        if self.available >= amount {
            self.available -= amount;
            Ok(())
        } else {
            Err(format!("insufficient funds to withdraw {amount}"))
        }
    }

    pub(crate) fn dispute(&mut self, disputed_tx: TxId) -> Result<(), String> {
        self.ensure_not_locked()?;

        if let Some(deposit_amount) = self.accepted_deposits.get(&disputed_tx) {
            if self.available >= *deposit_amount {
                let disputed_amount = self
                    .accepted_deposits
                    .remove(&disputed_tx)
                    .expect("presence checked above");
                self.available -= disputed_amount;
                self.held += disputed_amount;
                self.disputed_deposits.insert(disputed_tx, disputed_amount);
                Ok(())
            } else {
                Err("the funds of the disputed deposit were already withdrawn".to_string())
            }
        } else {
            Err("dispute referencing unknown transaction".to_string())
        }
    }

    pub(crate) fn resolve(&mut self, resolved_tx: TxId) -> Result<(), String> {
        self.ensure_not_locked()?;

        if let Some(resolved_amount) = self.disputed_deposits.remove(&resolved_tx) {
            debug_assert!(
                self.held_funds() >= resolved_amount,
                "internal logic error: held funds too low during resolve"
            );
            self.held -= resolved_amount;
            self.available += resolved_amount;
            self.accepted_deposits.insert(resolved_tx, resolved_amount);
            Ok(())
        } else {
            Err("resolve referencing unknown/undisputed transaction".to_string())
        }
    }

    pub(crate) fn chargeback(&mut self, reverted_tx: TxId) -> Result<(), String> {
        self.ensure_not_locked()?;

        if let Some(reverted_amount) = self.disputed_deposits.remove(&reverted_tx) {
            debug_assert!(
                self.held_funds() >= reverted_amount,
                "internal logic error: held funds too low during chargeback"
            );
            self.held -= reverted_amount;
            self.locked = true;
            Ok(())
        } else {
            Err("chargeback referencing unknown/undisputed transaction".to_string())
        }
    }

    fn ensure_not_locked(&self) -> Result<(), String> {
        if self.locked {
            Err("account locked: transaction rejected".to_string())
        } else {
            Ok(())
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
