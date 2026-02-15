//! Scenario-based integration tests.
//! Combines multiple per-client scenarios with random parameters and interleaving
//! to verify correctness, client isolation, and ordering.

mod catalog;
mod scenario;

use proptest::prelude::*;
use rust_decimal::Decimal;
use scenario::{Scenario, assert_scenarios, interleave, run_process};

proptest! {
    #[test]
    fn interleaved_scenarios_produce_correct_results(
        // Bump the upper bound of the first tuple element when the catalog grows beyond 8 shapes.
        // Increase the range of the second tuple element to test more clients simultaneously.
        shape_indices in prop::collection::vec(0usize..19, 2..=6),
        // Increase the pool size if shapes need many random parameters or max scenarios grows.
        random_parameters in prop::collection::vec(1u64..100_000, 50),
        seed in any::<u64>(),
    ) {
        let catalog = catalog::all_shapes();

        // 1. Pick shapes from the catalog, clamping indices to catalog size
        // 2. Build concrete scenarios with unique client_ids and non-overlapping tx_id ranges
        let mut param_cursor = 0;
        let mut tx_id_offset = 1u32;
        let mut scenarios = Vec::new();

        for (i, &idx) in shape_indices.iter().enumerate() {
            let shape = &catalog[idx % catalog.len()];
            let client_id = (i + 1) as u16;
            let n = shape.num_random_parameters();

            // Pull random parameters from the pool, wrapping if we run out
            let params: Vec<Decimal> = (0..n)
                .map(|j| {
                    let raw = random_parameters[(param_cursor + j) % random_parameters.len()];
                    Decimal::new(raw as i64, 4) // e.g. 12345 -> 1.2345
                })
                .collect();
            param_cursor += n;

            let scenario = shape.build(client_id, tx_id_offset, &params);
            tx_id_offset += scenario.transactions.len() as u32;
            scenarios.push(scenario);
        }

        // 3. Build an order-preserving interleaving schedule from the seed
        let schedule = build_schedule(&scenarios, seed);

        // 4. Interleave into a single CSV and run the engine
        let csv = interleave(&scenarios, &schedule);
        let result = run_process(&csv);

        // 5. Assert every scenario's expectations
        assert_scenarios(&scenarios, &result);
    }
}

/// Builds a schedule that references each scenario index exactly as many times
/// as it has transactions, then shuffles it (preserving per-scenario order
/// via the cursor mechanism in `interleave`).
fn build_schedule(scenarios: &[Scenario], seed: u64) -> Vec<usize> {
    // Flat list: [0, 0, ..., 1, 1, ..., 2, ...]
    let mut schedule: Vec<usize> = scenarios
        .iter()
        .enumerate()
        .flat_map(|(i, s)| std::iter::repeat_n(i, s.transactions.len()))
        .collect();

    // Fisher-Yates shuffle with a simple LCG seeded by proptest
    let mut rng = seed;
    for i in (1..schedule.len()).rev() {
        rng = rng
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let j = (rng >> 33) as usize % (i + 1);
        schedule.swap(i, j);
    }

    schedule
}

// ---------------------------------------------------------------------------
// Tests for the test infrastructure itself
// ---------------------------------------------------------------------------

mod tests {
    use super::*;
    use catalog::{SingleDeposit, TwoDeposits};
    use rust_decimal_macros::dec;
    use scenario::ScenarioShape;

    /// Convenience: build a SingleDeposit scenario with a given client, offset, and amount.
    fn single(client_id: u16, offset: u32, amount: Decimal) -> Scenario {
        SingleDeposit.build(client_id, offset, &[amount])
    }

    /// Convenience: build a TwoDeposits scenario with a given client, offset, and amounts.
    fn two(client_id: u16, offset: u32, amounts: [Decimal; 2]) -> Scenario {
        TwoDeposits.build(client_id, offset, &amounts)
    }

    // -- interleave -----------------------------------------------------------

    #[test]
    fn interleave_two_scenarios_ordered() {
        let scenarios = [
            single(1, 0, dec!(1.0)),           // 1 tx row
            two(2, 1, [dec!(2.0), dec!(3.0)]), // 2 tx rows
        ];
        let schedule = [0, 1, 1];

        let csv = interleave(&scenarios, &schedule);
        let lines: Vec<&str> = csv.lines().collect();

        assert_eq!(lines[0], "type, client, tx, amount");
        assert_eq!(lines[1], scenarios[0].transactions[0]); // client 1's deposit
        assert_eq!(lines[2], scenarios[1].transactions[0]); // client 2's first deposit
        assert_eq!(lines[3], scenarios[1].transactions[1]); // client 2's second deposit
        assert_eq!(lines.len(), 4);
    }

    #[test]
    fn interleave_preserves_per_scenario_order() {
        let scenarios = [
            two(1, 0, [dec!(1.0), dec!(2.0)]),
            two(2, 2, [dec!(3.0), dec!(4.0)]),
        ];
        let schedule = [1, 0, 0, 1];

        let csv = interleave(&scenarios, &schedule);
        let lines: Vec<&str> = csv.lines().skip(1).collect(); // skip header

        let s0_first = lines
            .iter()
            .position(|l| *l == scenarios[0].transactions[0])
            .unwrap();
        let s0_second = lines
            .iter()
            .position(|l| *l == scenarios[0].transactions[1])
            .unwrap();
        assert!(s0_first < s0_second, "client 1's rows must stay in order");

        let s1_first = lines
            .iter()
            .position(|l| *l == scenarios[1].transactions[0])
            .unwrap();
        let s1_second = lines
            .iter()
            .position(|l| *l == scenarios[1].transactions[1])
            .unwrap();
        assert!(s1_first < s1_second, "client 2's rows must stay in order");
    }

    #[test]
    fn interleave_single_scenario() {
        let s = two(1, 0, [dec!(1.0), dec!(2.0)]);
        let schedule = [0, 0];

        let csv = interleave(&[s], &schedule);
        let lines: Vec<&str> = csv.lines().collect();

        assert_eq!(lines.len(), 3); // header + 2 rows
    }

    #[test]
    #[should_panic(expected = "schedule references more transactions")]
    fn interleave_panics_on_overrun() {
        let s = single(1, 0, dec!(1.0)); // 1 tx
        let schedule = [0, 0]; // pulls 2
        interleave(&[s], &schedule);
    }

    #[test]
    #[should_panic(expected = "schedule only pulled")]
    fn interleave_panics_on_underrun() {
        let s = two(1, 0, [dec!(1.0), dec!(2.0)]); // 2 txs
        let schedule = [0]; // pulls 1
        interleave(&[s], &schedule);
    }

    // -- build_schedule -------------------------------------------------------

    #[test]
    fn build_schedule_has_correct_counts() {
        let s0 = single(1, 0, dec!(1.0)); // 1 tx
        let s1 = two(2, 1, [dec!(2.0), dec!(3.0)]); // 2 txs
        let scenarios = [s0, s1];

        let schedule = build_schedule(&scenarios, 42);

        let count_0 = schedule.iter().filter(|&&i| i == 0).count();
        let count_1 = schedule.iter().filter(|&&i| i == 1).count();
        assert_eq!(count_0, 1);
        assert_eq!(count_1, 2);
        assert_eq!(schedule.len(), 3);
    }

    #[test]
    fn build_schedule_is_deterministic() {
        let make = || [single(1, 0, dec!(1.0)), two(2, 1, [dec!(2.0), dec!(3.0)])];

        let a = build_schedule(&make(), 12345);
        let b = build_schedule(&make(), 12345);
        assert_eq!(a, b);
    }

    #[test]
    fn build_schedule_different_seeds_differ() {
        let make = || {
            [
                two(1, 0, [dec!(1.0), dec!(2.0)]),
                two(2, 2, [dec!(3.0), dec!(4.0)]),
                two(3, 4, [dec!(5.0), dec!(6.0)]),
                two(4, 6, [dec!(7.0), dec!(8.0)]),
            ]
        };

        let a = build_schedule(&make(), 1);
        let b = build_schedule(&make(), 2);
        // 8 elements, different seeds — astronomically unlikely to match
        assert_ne!(a, b);
    }

    #[test]
    fn build_schedule_single_scenario() {
        let s = two(1, 0, [dec!(1.0), dec!(2.0)]);
        let schedule = build_schedule(&[s], 999);
        // Only one scenario, all entries must be 0
        assert_eq!(schedule, vec![0, 0]);
    }

    // -- full roundtrip -------------------------------------------------------

    #[test]
    fn single_deposit_roundtrip() {
        let scenarios = [single(1, 0, dec!(5.0))];

        let csv = interleave(&scenarios, &[0]);
        let result = run_process(&csv);
        assert_scenarios(&scenarios, &result);
    }

    #[test]
    fn two_scenarios_roundtrip() {
        let scenarios = [single(1, 0, dec!(3.5)), two(2, 1, [dec!(1.0), dec!(2.0)])];
        let schedule = build_schedule(&scenarios, 42);

        let csv = interleave(&scenarios, &schedule);
        let result = run_process(&csv);
        assert_scenarios(&scenarios, &result);
    }
}
