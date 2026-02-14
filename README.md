# tx-engine-rs

A high-performance transaction processing engine in Rust. Ingests a stream of financial transactions from CSV, processes them in memory, and outputs the final state of all client accounts to STDOUT.

## Usage

TODO

## Assumptions

- **Zero-amount deposits are rejected.** A deposit of `0.0` has no effect on account balances but would still consume memory when stored for dispute resolution. These are treated as invalid input.

- **Erroneous transactions are skipped, not fatal.** Errors in the input CSV are handled per transaction — an invalid or malformed row is reported to the caller and otherwise ignored. Processing continues with the remaining transactions. This aligns with safely ignoring nonsensical operations regarding, e.g., disputes referencing non-existing transactions.

## Design Decisions

### No async runtime (yet)

The processing workload is CPU-bound and synchronous — workers receive transactions and update in-memory balances with nothing to `await`. Adding an async runtime (`tokio`) would introduce compile-time overhead without benefit. The domain logic is kept purely synchronous, making it straightforward to integrate with an async runtime later.

### Money representation: `Decimal` over `u64`

The two main candidates for representing monetary values are `u64` (storing the smallest unit, e.g., ten-thousandths) and `rust_decimal::Decimal`. `u64` is more compact and inherently non-negative — which fits this domain, since balances should never go negative by design. However, `Decimal` offers easier parsing from the CSV input format and simpler formatting on output, reducing boilerplate at this stage. Since all monetary fields are accessed through a type alias, switching to `u64` later is a low-cost optimization if needed.

### Caller-defined callbacks for success and failure

The library does not prescribe how successes or errors should be handled. Instead, the `process` entry point takes two callbacks:

- **`on_error`** — invoked for every transaction that cannot be processed. The caller can log, collect, count, or abort.
- **`on_success`** — invoked with a reference to each successfully applied transaction. Useful for logging, metrics, publishing events, or progress tracking.

This offers maximal flexibility and keeps the library agnostic about side effects. The design was also chosen with a multi-threaded architecture in mind: each worker thread can send successes and errors through channels to centralized handlers, without requiring any change to the library's API.

### No timestamps on transactions or accounts

Timestamps were considered for transactions (for auditing and enabling dispute-window-based eviction) and for accounts (`last_updated`). Both were deferred: the input format provides no event time, so timestamps would reflect processing time only — which is near-identical across a batch run and carries little information. Account-level `last_updated` adds a write on every operation for a field not consumed by the output. In a streaming or real-time system, event-time timestamps become valuable and can be added without changing the processing logic.

## Error Handling

The engine is designed to process large, potentially messy CSV inputs without aborting on the first bad row. Errors are categorised into two kinds:

- **CSV-level errors** — malformed rows that the `csv` crate cannot deserialize (e.g. missing columns, unparseable numbers).
- **Validation errors** — well-formed rows that violate domain rules (e.g. a deposit with a negative or zero amount, an unknown transaction type). These carry structured context: the `client_id`, `tx_id`, and a human-readable message.

Rather than choosing a fixed error policy inside the library, the `process` entry point accepts a caller-supplied callback (`on_error: impl FnMut(Error)`) that is invoked for every problematic transaction. The transaction is then skipped and processing continues.

This keeps the library agnostic about what "handling an error" means — the caller decides. In the included binary, we simply log warnings:

```rust
fn handle_tx_error(error: Error) {
    tracing::warn!("{error}");
}
```

To change the behaviour, replace or extend this function.

## Testing

Tests are run using [cargo-nextest](https://nexte.st/).

**Install nextest:**

```bash
cargo install cargo-nextest --locked
```

**Run all tests:**

```bash
cargo nextest run
```

## CI

Every pull request against `main` runs a GitHub Actions pipeline that enforces:

- **Formatting** — `cargo fmt --all --check` ensures consistent style.
- **Dependency audit** — `cargo deny check advisories` flags known vulnerabilities in dependencies.
- **Tests** — `cargo nextest run` runs the full test suite.
- **Coverage** — `cargo llvm-cov nextest --fail-under-lines 90` enforces a minimum of 90 % line coverage.

The pipeline definition lives in `.github/workflows/ci.yml`.

## Future Work

- **Transaction timestamps & dispute windows:** In a streaming system, transactions could carry event-time timestamps, enabling eviction of old transactions that are past their dispute window — reducing memory usage in long-running deployments.


---
**AI Usage Declaration:** This project utilized generative AI tools (Cursor) for architectural brainstorming and boilerplate generation. All code and logic were reviewed, tested, and validated by the author to ensure correctness and security. For a detailed log, see [AI_USAGE.md](./AI_USAGE.md).