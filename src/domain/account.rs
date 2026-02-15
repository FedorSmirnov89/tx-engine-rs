//! Module defining the domain types related to the representation of the client account

use std::collections::HashMap;

use crate::domain::{Deposit, Money, TxId};

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

    pub(crate) fn available_funds(&self) -> Money {
        self.available
    }
    pub(crate) fn held_funds(&self) -> Money {
        self.held
    }
    pub(crate) fn is_locked(&self) -> bool {
        self.locked
    }
}
