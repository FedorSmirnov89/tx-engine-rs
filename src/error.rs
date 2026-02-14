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
}

pub(crate) fn validation_error(client_id: u16, tx_id: u32, message: impl Into<String>) -> Error {
    Error::Validation {
        client_id,
        tx_id,
        message: message.into(),
    }
}
