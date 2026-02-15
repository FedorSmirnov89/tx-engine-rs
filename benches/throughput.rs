//! Criterion benchmark measuring end-to-end throughput of the transaction engine.
//!
//! Prerequisite: generate the benchmark fixture first:
//!   cargo nextest run --run-ignored only generate_benchmark_fixture

use std::path::PathBuf;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use tx_engine_rs::{AccountRecord, Error, TransactionRecord, process};

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("data")
        .join("benchmark.csv")
}

fn bench_process(c: &mut Criterion) {
    let csv_bytes = std::fs::read(fixture_path()).expect(
        "benchmark fixture not found — run: \
         cargo nextest run --run-ignored only generate_benchmark_fixture'",
    );

    let row_count = csv_bytes.iter().filter(|&&b| b == b'\n').count() - 1; // minus header

    let mut group = c.benchmark_group("process");
    group.measurement_time(std::time::Duration::from_secs(25));
    group.throughput(Throughput::Elements(row_count as u64));

    group.bench_function(BenchmarkId::new("sequential", row_count), |b| {
        b.iter(|| {
            let accounts: Vec<AccountRecord> = process(
                csv_bytes.as_slice(),
                |_: Error| {},
                |_: TransactionRecord| {},
            )
            .collect();
            criterion::black_box(accounts);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_process);
criterion_main!(benches);
