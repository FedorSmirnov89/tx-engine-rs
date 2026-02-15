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
            name: "SingleDeposit",
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
            name: "TwoDeposits",
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
            name: "DepositsWithInvalidAmounts",
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

/// Deposit then withdraw a smaller amount. Both succeed.
/// params: [0] = deposit - withdrawal, [1] = withdrawal (deposit always >= withdrawal)
pub struct DepositThenWithdraw;

impl ScenarioShape for DepositThenWithdraw {
    fn num_random_parameters(&self) -> usize {
        2
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let deposit = random_parameters[0] + random_parameters[1];
        let withdrawal = random_parameters[1];
        let remaining = deposit - withdrawal; // = random_parameters[0], always positive

        let tx_dep = tx_id_offset + 1;
        let tx_wdr = tx_id_offset + 2;

        Scenario {
            name: "DepositThenWithdraw",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {withdrawal}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: remaining,
                held: Decimal::ZERO,
                total: remaining,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr],
            expected_errors: vec![],
        }
    }
}

/// Deposit a large amount, then withdraw twice. All three succeed.
/// params: [0] = extra remaining, [1] = first withdrawal, [2] = second withdrawal
/// deposit = params[0] + params[1] + params[2], so both withdrawals fit.
pub struct DepositThenTwoWithdrawals;

impl ScenarioShape for DepositThenTwoWithdrawals {
    fn num_random_parameters(&self) -> usize {
        3
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let wdr_a = random_parameters[1];
        let wdr_b = random_parameters[2];
        let deposit = random_parameters[0] + wdr_a + wdr_b;
        let remaining = deposit - wdr_a - wdr_b;

        let tx_dep = tx_id_offset + 1;
        let tx_wdr_a = tx_id_offset + 2;
        let tx_wdr_b = tx_id_offset + 3;

        Scenario {
            name: "DepositThenTwoWithdrawals",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr_a}, {wdr_a}"),
                format!("withdrawal, {client_id}, {tx_wdr_b}, {wdr_b}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: remaining,
                held: Decimal::ZERO,
                total: remaining,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr_a, tx_wdr_b],
            expected_errors: vec![],
        }
    }
}

/// Withdraw with no funds (fails), then deposit, then withdraw within balance (succeeds).
/// params: [0] = failed withdrawal amount, [1] = deposit, [2] = valid withdrawal (< deposit)
/// deposit = params[1] + params[2], valid withdrawal = params[2], so it always fits.
pub struct OverdraftThenDepositThenWithdraw;

impl ScenarioShape for OverdraftThenDepositThenWithdraw {
    fn num_random_parameters(&self) -> usize {
        3
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let overdraft = random_parameters[0];
        let deposit = random_parameters[1] + random_parameters[2];
        let valid_wdr = random_parameters[2];
        let remaining = deposit - valid_wdr; // = random_parameters[1]

        let tx_overdraft = tx_id_offset + 1;
        let tx_dep = tx_id_offset + 2;
        let tx_wdr = tx_id_offset + 3;

        Scenario {
            name: "OverdraftThenDepositThenWithdraw",
            client_id,
            transactions: vec![
                format!("withdrawal, {client_id}, {tx_overdraft}, {overdraft}"),
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {valid_wdr}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: remaining,
                held: Decimal::ZERO,
                total: remaining,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr],
            expected_errors: vec![tx_overdraft],
        }
    }
}

/// Deposit then attempt to withdraw more than the balance. The withdrawal fails.
/// params: [0] = deposit, [1] = extra (withdrawal = deposit + extra, always > deposit)
pub struct DepositThenOverdraft;

impl ScenarioShape for DepositThenOverdraft {
    fn num_random_parameters(&self) -> usize {
        2
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let deposit = random_parameters[0];
        let withdrawal = random_parameters[0] + random_parameters[1]; // always > deposit

        let tx_dep = tx_id_offset + 1;
        let tx_wdr = tx_id_offset + 2;

        Scenario {
            name: "DepositThenOverdraft",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {withdrawal}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: deposit,
                held: Decimal::ZERO,
                total: deposit,
                locked: false,
            },
            expected_successes: vec![tx_dep],
            expected_errors: vec![tx_wdr],
        }
    }
}

/// Deposit then withdraw the exact same amount. Both succeed, balance = 0.
/// params: [0] = the amount for both deposit and withdrawal
pub struct ExactBalanceWithdrawal;

impl ScenarioShape for ExactBalanceWithdrawal {
    fn num_random_parameters(&self) -> usize {
        1
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount = random_parameters[0];

        let tx_dep = tx_id_offset + 1;
        let tx_wdr = tx_id_offset + 2;

        Scenario {
            name: "ExactBalanceWithdrawal",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {amount}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {amount}"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: Decimal::ZERO,
                held: Decimal::ZERO,
                total: Decimal::ZERO,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr],
            expected_errors: vec![],
        }
    }
}

/// Deposit then dispute the deposit. Funds move from available to held.
/// params: [0] = deposit amount
pub struct DepositThenDispute;

impl ScenarioShape for DepositThenDispute {
    fn num_random_parameters(&self) -> usize {
        1
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount = random_parameters[0];
        let tx_dep = tx_id_offset + 1;

        Scenario {
            name: "DepositThenDispute",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {amount}"),
                format!("dispute, {client_id}, {tx_dep},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: Decimal::ZERO,
                held: amount,
                total: amount,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_dep],
            expected_errors: vec![],
        }
    }
}

/// Deposit then dispute a nonexistent tx. The dispute fails, balance unchanged.
/// params: [0] = deposit amount
pub struct DisputeNonexistentTx;

impl ScenarioShape for DisputeNonexistentTx {
    fn num_random_parameters(&self) -> usize {
        1
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount = random_parameters[0];
        let tx_dep = tx_id_offset + 1;
        let tx_fake = tx_id_offset + 2;

        Scenario {
            name: "DisputeNonexistentTx",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {amount}"),
                format!("dispute, {client_id}, {tx_fake},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: amount,
                held: Decimal::ZERO,
                total: amount,
                locked: false,
            },
            expected_successes: vec![tx_dep],
            expected_errors: vec![tx_fake],
        }
    }
}

/// Deposit, withdraw, then dispute the withdrawal. The dispute fails — only deposits are disputable.
/// params: [0] = remaining after withdrawal, [1] = withdrawal amount
/// deposit = [0] + [1], withdrawal = [1]
pub struct DisputeAWithdrawal;

impl ScenarioShape for DisputeAWithdrawal {
    fn num_random_parameters(&self) -> usize {
        2
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let withdrawal = random_parameters[1];
        let deposit = random_parameters[0] + withdrawal;
        let remaining = deposit - withdrawal;

        let tx_dep = tx_id_offset + 1;
        let tx_wdr = tx_id_offset + 2;

        Scenario {
            name: "DisputeAWithdrawal",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {withdrawal}"),
                format!("dispute, {client_id}, {tx_wdr},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: remaining,
                held: Decimal::ZERO,
                total: remaining,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr],
            expected_errors: vec![tx_wdr],
        }
    }
}

/// Deposit, withdraw part of it, then dispute the deposit. The dispute fails because
/// available < disputed amount (funds have been partially spent).
/// params: [0] = remaining available after withdrawal, [1] = withdrawal amount
/// deposit = [0] + [1]
pub struct DisputeInsufficientFunds;

impl ScenarioShape for DisputeInsufficientFunds {
    fn num_random_parameters(&self) -> usize {
        2
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let remaining = random_parameters[0];
        let withdrawal = random_parameters[1];
        let deposit = remaining + withdrawal;

        let tx_dep = tx_id_offset + 1;
        let tx_wdr = tx_id_offset + 2;

        Scenario {
            name: "DisputeInsufficientFunds",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {deposit}"),
                format!("withdrawal, {client_id}, {tx_wdr}, {withdrawal}"),
                format!("dispute, {client_id}, {tx_dep},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: remaining,
                held: Decimal::ZERO,
                total: remaining,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_wdr],
            expected_errors: vec![tx_dep],
        }
    }
}

/// Deposit then dispute the same tx twice. The second dispute fails — already under dispute.
/// params: [0] = deposit amount
pub struct DoubleDispute;

impl ScenarioShape for DoubleDispute {
    fn num_random_parameters(&self) -> usize {
        1
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let amount = random_parameters[0];
        let tx_dep = tx_id_offset + 1;

        Scenario {
            name: "DoubleDispute",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep}, {amount}"),
                format!("dispute, {client_id}, {tx_dep},"),
                format!("dispute, {client_id}, {tx_dep},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: Decimal::ZERO,
                held: amount,
                total: amount,
                locked: false,
            },
            expected_successes: vec![tx_dep, tx_dep],
            expected_errors: vec![tx_dep],
        }
    }
}

/// Two deposits, then dispute only the first. Only the first deposit's funds are held.
/// params: [0] = first deposit, [1] = second deposit
pub struct TwoDepositsDisputeFirst;

impl ScenarioShape for TwoDepositsDisputeFirst {
    fn num_random_parameters(&self) -> usize {
        2
    }

    fn build(&self, client_id: u16, tx_id_offset: u32, random_parameters: &[Decimal]) -> Scenario {
        let first = random_parameters[0];
        let second = random_parameters[1];

        let tx_dep_1 = tx_id_offset + 1;
        let tx_dep_2 = tx_id_offset + 2;

        Scenario {
            name: "TwoDepositsDisputeFirst",
            client_id,
            transactions: vec![
                format!("deposit, {client_id}, {tx_dep_1}, {first}"),
                format!("deposit, {client_id}, {tx_dep_2}, {second}"),
                format!("dispute, {client_id}, {tx_dep_1},"),
            ],
            expected_account: AccountRecord {
                client: client_id,
                available: second,
                held: first,
                total: first + second,
                locked: false,
            },
            expected_successes: vec![tx_dep_1, tx_dep_2, tx_dep_1],
            expected_errors: vec![],
        }
    }
}

/// Returns all available scenario shapes.
pub fn all_shapes() -> Vec<Box<dyn ScenarioShape>> {
    vec![
        Box::new(SingleDeposit),
        Box::new(TwoDeposits),
        Box::new(DepositsWithInvalidAmounts),
        Box::new(DepositThenWithdraw),
        Box::new(DepositThenTwoWithdrawals),
        Box::new(OverdraftThenDepositThenWithdraw),
        Box::new(DepositThenOverdraft),
        Box::new(ExactBalanceWithdrawal),
        Box::new(DepositThenDispute),
        Box::new(DisputeNonexistentTx),
        Box::new(DisputeAWithdrawal),
        Box::new(DisputeInsufficientFunds),
        Box::new(DoubleDispute),
        Box::new(TwoDepositsDisputeFirst),
        // ... add more as transaction types are implemented
    ]
}
