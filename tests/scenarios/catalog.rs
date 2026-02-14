//! Library of hand-crafted scenario shapes.
//! Each shape defines a sequence of transaction types and a formula for the expected outcome.
//! Add new shapes here as new transaction types are implemented.

use rust_decimal::Decimal;
use tx_engine_rs::AccountRecord;

use super::scenario::{Scenario, ScenarioShape};

/// Deposit a single amount.
/// Expected: available = amount, held = 0, locked = false
pub struct SingleDeposit;

impl ScenarioShape for SingleDeposit {
    fn num_random_parameters(&self) -> usize {
        1
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount = random_parameters[0];
        let tx_id = tx_id_offset + 1;

        Scenario {
            client_id,
            transactions: vec![format!("deposit, {client_id}, {tx_id}, {amount}")],
            expected_account: AccountRecord {
                client: client_id,
                available: amount,
                held: Decimal::ZERO,
                total: amount,
                locked: false,
            },
            expected_successes: vec![tx_id],
            expected_errors: vec![],
        }
    }
}

/// Two valid deposits.
/// We expect to have the sum of amounts as total. Both TXs should be valid
pub struct TwoDeposits;

impl ScenarioShape for TwoDeposits {
    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount_a = random_parameters[0];
        let amount_b = random_parameters[1];

        Scenario {
            client_id,
            transactions: vec![
                format!(
                    "deposit, {client_id}, {tx_id}, {amount_a}",
                    tx_id = tx_id_offset + 1,
                ),
                format!(
                    "deposit, {client_id}, {tx_id}, {amount_b}",
                    tx_id = tx_id_offset + 2,
                ),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: amount_a + amount_b,
                held: Decimal::ZERO,
                total: amount_a + amount_b,
                locked: false,
            },

            expected_successes: vec![tx_id_offset + 1, tx_id_offset + 2],
            expected_errors: vec![],
        }
    }

    fn num_random_parameters(&self) -> usize {
        2
    }
}

/// Three deposits: one valid (positive random), one zero, one negative (negated random).
/// Only the positive deposit should succeed; the zero and negative ones should error.
pub struct DepositsWithInvalidAmounts;

impl ScenarioShape for DepositsWithInvalidAmounts {
    fn num_random_parameters(&self) -> usize {
        2 // [0] = valid deposit amount, [1] = base for the negative deposit
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let valid_amount = random_parameters[0];
        let negative_amount = -random_parameters[1];

        let tx_valid = tx_id_offset + 1;
        let tx_zero = tx_id_offset + 2;
        let tx_negative = tx_id_offset + 3;

        Scenario {
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_valid}, {valid_amount}"),
                format!("deposit, {client_id}, {tx_zero}, 0"),
                format!("deposit, {client_id}, {tx_negative}, {negative_amount}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: valid_amount,
                held: Decimal::ZERO,
                total: valid_amount,
                locked: false,
            },
            expected_successes: vec![tx_valid],
            expected_errors: vec![tx_zero, tx_negative],
        }
    }
}

/// Returns all available scenario shapes.
pub fn all_shapes() -> Vec<Box<dyn ScenarioShape>> {
    vec![
        Box::new(SingleDeposit),
        Box::new(TwoDeposits),
        Box::new(DepositsWithInvalidAmounts),
        // ... add more as transaction types are implemented
    ]
}
