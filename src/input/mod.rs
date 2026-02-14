//! Module defining the parsing logic used to convert the user-provided input into validated domain types that can be provided to the core logic of the engine.

use std::io::Read;

use anyhow::{Result, bail};
use rust_decimal::Decimal;
use serde::Deserialize;

use crate::domain::{ClientId, Deposit, Transaction, TxId};

#[cfg(test)]
mod tests;

/// Parses the data provided by the reader and returns an iterator over the parsing results
pub(crate) fn parse_transactions(reader: impl Read) -> impl Iterator<Item = Result<Transaction>> {
    let csv_reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(reader);

    csv_reader
        .into_deserialize::<RawTransaction>()
        .map(|result| {
            let raw = result?;
            Transaction::try_from(raw)
        })
}

// Intermediate type mirroring the CSV columns
#[derive(Deserialize)]
struct RawTransaction {
    #[serde(rename = "type")]
    tx_type: String,
    client: u16,
    tx: u32,
    amount: Option<Decimal>,
}

impl TryFrom<RawTransaction> for Transaction {
    type Error = anyhow::Error;

    fn try_from(raw: RawTransaction) -> Result<Self, Self::Error> {
        let RawTransaction {
            tx_type,
            client,
            tx,
            amount,
        } = raw;
        match tx_type.as_str() {
            "deposit" => {
                let client_id = ClientId::new(client);
                let tx_id = TxId::new(tx);
                let Some(amount) = amount else {
                    bail!("no amount provided for deposit transaction with ID {tx}")
                };
                Ok(Transaction::Deposit(Deposit::new(
                    client_id, tx_id, amount,
                )?))
            }
            other => bail!("unknown transaction type: {other}"),
        }
    }
}
