//! Module for the script deterministically generating a large input used for an e2e test

use crate::{
    generate::{build_catalog_scenarios, data_dir, write_expected_csv, write_text},
    scenarios::{build_schedule, scenario::interleave},
};

const SEED: u64 = 20260215; // stable seed — today's date

/// Generates the representative E2E fixture.
/// All 29 shapes × 4 repetitions = 116 clients, ~450 tx rows.
/// Produces both input CSV and expected output CSV.
///
/// The files are checked in, but can be (re)generated using
/// `cargo nextest run --run-ignored only generate_representative_fixture`
#[test]
#[ignore]
fn generate_representative_fixture() {
    let scenarios = build_catalog_scenarios(4);
    let schedule = build_schedule(&scenarios, SEED);
    let csv = interleave(&scenarios, &schedule);

    let dir = data_dir();
    write_text(&csv, &dir.join("representative.csv"));
    write_expected_csv(&scenarios, &dir.join("representative_expected.csv"));

    eprintln!(
        "Representative fixture: {} clients, {} tx rows",
        scenarios.len(),
        schedule.len(),
    );
}
