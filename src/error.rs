//! Module defining the errors which are exposed to the users of the crate

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Invalid CSV
    #[error("CSV error: {0}")]
    Csv(#[from] csv::Error),

    /// Valid CSV violating domain invariants, e.g., a deposit with a negative amount
    #[error("validation error — client: {client_id}, tx: {tx_id}: {message}")]
    Validation {
        client_id: u16,
        tx_id: u32,
        message: String,
    },

    /// Valid CSV satisfying domain invariants, but inconsistent with the current state (e.g., withdrawal exceeding the available amount)
    #[error("processing conflict — client: {client_id}, tx: {tx_id}: {message}")]
    Processing {
        client_id: u16,
        tx_id: u32,
        message: String,
    },
}

pub(crate) fn validation_error(
    client_id: impl Into<u16>,
    tx_id: impl Into<u32>,
    message: impl Into<String>,
) -> Error {
    Error::Validation {
        client_id: client_id.into(),
        tx_id: tx_id.into(),
        message: message.into(),
    }
}

pub(crate) fn processing_error(
    client_id: impl Into<u16>,
    tx_id: impl Into<u32>,
    message: impl Into<String>,
) -> Error {
    Error::Processing {
        client_id: client_id.into(),
        tx_id: tx_id.into(),
        message: message.into(),
    }
}
