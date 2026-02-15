//! Module for the core logic of the engine

use std::collections::HashMap;

use crate::domain::{AccountState, ClientId};

mod logic;
mod orchestration;

pub(crate) use orchestration::{process_transactions, process_transactions_parallel};

type Accounts = HashMap<ClientId, AccountState>;
