//! Integration tests testing against the actual crate binary and reading from and writing to a file: Test the full E2E path.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn two_deposits() {
    let input_path = fixture_path("two_deposits.csv");
    let expected = std::fs::read_to_string(fixture_path("two_deposits_expected.csv"))
        .expect("failed to read expected output fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_tx-engine-rs"))
        .arg(&input_path)
        .output()
        .expect("failed to execute binary");

    assert!(
        output.status.success(),
        "binary exited with non-zero status.\nstderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("binary output was not valid UTF-8");

    assert_eq!(normalize_csv(&stdout), normalize_csv(&expected));
}

#[test]
fn representative_e2e() {
    let input_path = fixture_path("representative.csv");
    let expected = std::fs::read_to_string(fixture_path("representative_expected.csv"))
        .expect("failed to read expected output fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_tx-engine-rs"))
        .arg(&input_path)
        .output()
        .expect("failed to execute binary");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(normalize_csv(&stdout), normalize_csv(&expected));
}

/// Returns the absolute path to a test fixture file in `tests/data/`.
fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join(name)
}

/// Normalizes CSV for comparison, making comparison order-independent.
/// Used only to compare the output with the expected file since the order in our output is non-deterministic)
fn normalize_csv(raw: &str) -> String {
    let mut lines: Vec<String> = raw
        .lines()
        .map(|line| {
            line.split(',')
                .map(|cell| cell.trim())
                .map(|cell| {
                    // Normalize decimal representations: "0.0000" → "0"
                    cell.parse::<rust_decimal::Decimal>()
                        .map(|d| d.normalize().to_string())
                        .unwrap_or_else(|_| cell.to_string())
                })
                .collect::<Vec<_>>()
                .join(",")
        })
        .filter(|line| !line.is_empty())
        .collect();

    if lines.len() <= 1 {
        return lines.join("\n");
    }

    let header = lines.remove(0);
    lines.sort();

    std::iter::once(header)
        .chain(lines)
        .collect::<Vec<_>>()
        .join("\n")
}
