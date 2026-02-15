//! Module defining the parsing logic used to convert the user-provided input into validated domain types that can be provided to the core logic of the engine.

use std::io::Read;

use rust_decimal::Decimal;
use serde::Deserialize;

use crate::domain::{
    Chargeback, ClientId, Deposit, Dispute, Resolve, Transaction, TxId, Withdrawal,
};
use crate::error::{Error, validation_error};

pub(crate) const TYPE_KW_DEPOSIT: &str = "deposit";
pub(crate) const TYPE_KW_WITHDRAWAL: &str = "withdrawal";
pub(crate) const TYPE_KW_DISPUTE: &str = "dispute";
pub(crate) const TYPE_KW_RESOLVE: &str = "resolve";
pub(crate) const TYPE_KW_CHARGEBACK: &str = "chargeback";

#[cfg(test)]
mod tests;

/// Parses the data provided by the reader and returns an iterator over the parsing results
pub(crate) fn parse_transactions(
    reader: impl Read,
) -> impl Iterator<Item = Result<Transaction, Error>> {
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
    type Error = crate::error::Error;

    fn try_from(raw: RawTransaction) -> Result<Self, Self::Error> {
        let RawTransaction {
            tx_type,
            client,
            tx,
            amount,
        } = raw;

        let client_id = ClientId::new(client);
        let tx_id = TxId::new(tx);

        match tx_type.as_str() {
            TYPE_KW_DEPOSIT => {
                let Some(amount) = amount else {
                    return Err(validation_error(
                        client,
                        tx,
                        "no amount provided for deposit",
                    ));
                };
                Ok(Transaction::Deposit(
                    Deposit::new(client_id, tx_id, amount)
                        .map_err(|msg| validation_error(client, tx, msg))?,
                ))
            }
            TYPE_KW_WITHDRAWAL => {
                let Some(amount) = amount else {
                    return Err(validation_error(
                        client,
                        tx,
                        "no amount provided for withdrawal",
                    ));
                };
                Ok(Transaction::Withdrawal(
                    Withdrawal::new(client_id, tx_id, amount)
                        .map_err(|msg| validation_error(client, tx, msg))?,
                ))
            }
            TYPE_KW_DISPUTE => {
                if amount.is_some() {
                    return Err(validation_error(
                        client_id,
                        tx_id,
                        "an amount must not be provided with a dispute transaction",
                    ));
                }
                Ok(Transaction::Dispute(Dispute::new(client_id, tx_id)))
            }
            TYPE_KW_RESOLVE => {
                if amount.is_some() {
                    return Err(validation_error(
                        client_id,
                        tx_id,
                        "an amount must not be provided with a resolve transaction",
                    ));
                }
                Ok(Transaction::Resolve(Resolve::new(client_id, tx_id)))
            }
            TYPE_KW_CHARGEBACK => {
                if amount.is_some() {
                    return Err(validation_error(
                        client_id,
                        tx_id,
                        "an amount must not be provided with a chargeback transaction",
                    ));
                }
                Ok(Transaction::Chargeback(Chargeback::new(client_id, tx_id)))
            }
            other => Err(validation_error(
                client,
                tx,
                format!("unknown transaction type: {other}"),
            )),
        }
    }
}
