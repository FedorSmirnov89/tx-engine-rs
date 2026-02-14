//! Defines the `Scenario` type and the logic for combining scenarios into a single CSV input.

use std::collections::HashMap;

use rust_decimal::Decimal;
use tx_engine_rs::{AccountRecord, Error, TransactionRecord};

/// A self-contained per-client test story.
pub struct Scenario {
    /// Unique client ID for this scenario
    pub client_id: u16,
    /// Ordered CSV rows (without header) for this client
    pub transactions: Vec<String>,
    /// The expected final account state after all transactions
    pub expected_account: AccountRecord,
    /// Transaction IDs that should be processed successfully
    pub expected_successes: Vec<u32>,
    /// Transaction IDs that should produce errors
    pub expected_errors: Vec<u32>,
}

/// A trait for scenario shapes that can be instantiated with random parameters.
pub trait ScenarioShape {
    /// Build a concrete scenario from generated random parameters and a client ID.
    /// The tx_id_offset ensures globally unique transaction IDs across scenarios.
    /// Random parameters are independent values from proptest — the shape's `build`
    /// method may combine or transform them to establish required relationships
    /// (e.g., ensuring a withdrawal exceeds a deposit).
    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario;

    /// How many random parameters this shape needs from proptest.
    fn num_random_parameters(&self) -> usize;
}

/// Takes multiple scenarios and produces a single CSV string with their transactions
/// interleaved in an order-preserving way according to the given schedule.
///
/// `schedule` is a sequence of scenario indices (e.g., [0, 1, 0, 0, 1])
/// indicating which scenario to pull the next transaction from.
/// Each index must appear exactly as many times as its scenario has transactions.
pub fn interleave(scenarios: &[Scenario], schedule: &[usize]) -> String {
    let mut cursors = vec![0usize; scenarios.len()];
    let mut rows = Vec::with_capacity(schedule.len() + 1);

    rows.push("type, client, tx, amount".to_string());

    for &idx in schedule {
        let scenario = &scenarios[idx];
        let cursor = &mut cursors[idx];
        assert!(
            *cursor < scenario.transactions.len(),
            "schedule references more transactions than scenario {idx} has"
        );
        rows.push(scenario.transactions[*cursor].clone());
        *cursor += 1;
    }

    // Verify every scenario was fully consumed
    for (i, (cursor, scenario)) in cursors.iter().zip(scenarios).enumerate() {
        assert_eq!(
            *cursor,
            scenario.transactions.len(),
            "scenario {i} has {} transactions but schedule only pulled {}",
            scenario.transactions.len(),
            cursor
        );
    }

    rows.join("\n")
}

/// Runs `process` and collects results keyed by client_id for easy assertion.
pub struct ProcessResult {
    pub accounts: HashMap<u16, AccountRecord>,
    pub successes: HashMap<u16, Vec<u32>>,
    pub errors: HashMap<u16, Vec<u32>>,
}

pub fn run_process(csv_input: &str) -> ProcessResult {
    let mut successes: HashMap<u16, Vec<u32>> = HashMap::new();
    let mut errors: HashMap<u16, Vec<u32>> = HashMap::new();

    let accounts: HashMap<u16, AccountRecord> = tx_engine_rs::process(
        csv_input.as_bytes(),
        |e| {
            if let Error::Validation {
                client_id, tx_id, ..
            } = &e
            {
                errors.entry(*client_id).or_default().push(*tx_id);
            }
        },
        |tx| {
            let (client, tx_id) = tx_record_fields(&tx);
            successes.entry(client).or_default().push(tx_id);
        },
    )
    .map(|a| (a.client, a))
    .collect();

    ProcessResult {
        accounts,
        successes,
        errors,
    }
}

/// Asserts that each scenario's expectations match the process result.
pub fn assert_scenarios(scenarios: &[Scenario], result: &ProcessResult) {
    for scenario in scenarios {
        let cid = scenario.client_id;

        // --- Account state ---
        let account = result
            .accounts
            .get(&cid)
            .unwrap_or_else(|| panic!("no account record for client {cid}"));
        assert_eq!(
            *account, scenario.expected_account,
            "account mismatch for client {cid}"
        );

        // --- Successful transaction IDs ---
        let mut actual_successes = result.successes.get(&cid).cloned().unwrap_or_default();
        actual_successes.sort();
        let mut expected_successes = scenario.expected_successes.clone();
        expected_successes.sort();
        assert_eq!(
            actual_successes, expected_successes,
            "success tx_id mismatch for client {cid}"
        );

        // --- Error transaction IDs ---
        let mut actual_errors = result.errors.get(&cid).cloned().unwrap_or_default();
        actual_errors.sort();
        let mut expected_errors = scenario.expected_errors.clone();
        expected_errors.sort();
        assert_eq!(
            actual_errors, expected_errors,
            "error tx_id mismatch for client {cid}"
        );
    }
}

/// Extracts (client, tx_id) from any TransactionRecord variant.
fn tx_record_fields(tx: &TransactionRecord) -> (u16, u32) {
    match tx {
        TransactionRecord::Deposit { client, tx, .. } => (*client, *tx),
    }
}
