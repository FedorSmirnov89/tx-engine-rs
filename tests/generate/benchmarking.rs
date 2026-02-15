//! Generates a large input file for criterion benchmarks.

use crate::{
    generate::{build_catalog_scenarios, data_dir, write_text},
    scenarios::{build_schedule, scenario::interleave},
};

const SEED: u64 = 20260215;

/// All 29 shapes × 2000 repetitions = 58 000 clients, ~200k tx rows.
/// Produces only an input CSV (no expected output — benchmarks measure throughput, not correctness).
///
/// Regenerate with:
/// `cargo nextest run --run-ignored only generate_benchmark_fixture`
#[test]
#[ignore]
fn generate_benchmark_fixture() {
    let scenarios = build_catalog_scenarios(2000);
    let schedule = build_schedule(&scenarios, SEED);
    let csv = interleave(&scenarios, &schedule);

    let dir = data_dir();
    write_text(&csv, &dir.join("benchmark.csv"));

    let row_count = csv.lines().count() - 1; // minus header
    eprintln!(
        "Benchmark fixture: {} clients, {row_count} tx rows, {:.1} MB",
        scenarios.len(),
        csv.len() as f64 / (1024.0 * 1024.0),
    );
}
