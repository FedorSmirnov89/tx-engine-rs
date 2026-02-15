//! This module contains scripts which are used to generate the inputs used for other tests or benchmarking. They are implemented as ignored tests to enable running them comfortably

use std::path::PathBuf;

use rust_decimal::Decimal;

use crate::scenarios::{catalog::all_shapes, scenario::Scenario};

mod benchmarking;
mod end_to_end;

fn data_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
}

fn write_text(content: &str, path: &std::path::Path) {
    std::fs::write(path, content)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
    eprintln!("  wrote {}", path.display());
}

fn write_expected_csv(scenarios: &[Scenario], path: &std::path::Path) {
    let mut wtr = csv::Writer::from_path(path)
        .unwrap_or_else(|e| panic!("failed to create {}: {e}", path.display()));
    for s in scenarios {
        wtr.serialize(&s.expected_account)
            .expect("failed to serialize expected account");
    }
    wtr.flush().expect("failed to flush expected CSV");
    eprintln!("  wrote {}", path.display());
}

/// Builds scenarios from the full catalog, repeated `reps` times.
/// Each repetition assigns unique client IDs and non-overlapping tx ID ranges.
fn build_catalog_scenarios(reps: usize) -> Vec<Scenario> {
    let shapes = all_shapes();
    let num_shapes = shapes.len();
    let total_clients = reps * num_shapes;
    assert!(
        total_clients <= u16::MAX as usize,
        "too many clients ({total_clients}) — reduce repetitions"
    );

    let mut scenarios = Vec::with_capacity(total_clients);
    let mut client_id = 1u16;
    let mut tx_id_offset = 1u32;

    for rep in 0..reps {
        for (shape_idx, shape) in shapes.iter().enumerate() {
            let n = shape.num_random_parameters();
            let params: Vec<Decimal> = (0..n)
                .map(|i| {
                    let seed = (rep as u64) * 1000 + (shape_idx as u64) * 100 + (i as u64) + 1;
                    fixed_param(seed)
                })
                .collect();

            let scenario = shape.build(client_id, tx_id_offset, &params);
            tx_id_offset += scenario.transactions.len() as u32;
            scenarios.push(scenario);
            client_id += 1;
        }
    }

    scenarios
}

/// Produces a positive decimal in [0.0001, 10.0000) from a seed.
fn fixed_param(seed: u64) -> Decimal {
    let raw = (seed % 99_999) + 1; // 1..=99_999
    Decimal::new(raw as i64, 4) // e.g. 12345 → 1.2345
}
