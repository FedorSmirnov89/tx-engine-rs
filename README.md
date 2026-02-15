# tx-engine-rs

A high-performance transaction processing engine in Rust. Ingests a stream of financial transactions from CSV, processes them in memory, and outputs the final state of all client accounts to STDOUT.

## Usage

**Build:**

```bash
cargo build --release
```

**Run:**

```bash
cargo run -- transactions.csv > accounts.csv
```

The engine reads a CSV file of transactions from the path given as the first argument and writes the resulting account states to STDOUT. Logs are written to STDERR so they don't interfere with the data output.

**Environment variables:**

| Variable     | Default  | Description                                      |
|--------------|----------|--------------------------------------------------|
| `RUST_LOG`   | `info`   | Log level filter (e.g. `debug`, `warn`, `off`)   |
| `LOG_FORMAT` | `pretty` | Log output format (`pretty` or `json`)           |

**Input format:**

```csv
type,client,tx,amount
deposit,1,1,1.0
deposit,2,2,2.0
deposit,1,3,2.0
withdrawal,1,4,1.5
dispute,1,3,
resolve,1,3,
```

**Output format:**

```csv
client,available,held,total,locked
1,1.5,0,1.5,false
2,2.0,0,2.0,false
```

## Assumptions

- **Zero-amount deposits are rejected.** A deposit of `0.0` has no effect on account balances but would still consume memory when stored for dispute resolution. These are treated as invalid input.

- **Zero-amount withdrawals are rejected.** Same reasoning as zero-amount deposits — no effect on balances, waste of processing and storage.

- **Transaction type keywords are lowercase.** The input is expected to use exact lowercase keywords (`deposit`, `withdrawal`, `dispute`, `resolve`, `chargeback`). Mixed or uppercase variants are treated as unknown types and rejected.

- **Erroneous transactions are skipped, not fatal.** Errors in the input CSV are handled per transaction — an invalid or malformed row is reported to the caller and otherwise ignored. Processing continues with the remaining transactions. This aligns with safely ignoring nonsensical operations regarding, e.g., disputes referencing non-existing transactions.

- **Only deposits can be disputed.** A dispute on a withdrawal is ignored. If a client is unhappy with a withdrawal, the recourse is with the destination they withdrew to — our system has no mechanism to "undo" funds that have already left. Conversely, disputing a deposit (incoming funds) is the standard chargeback model: the sender claims the transfer was erroneous, and we must act to prevent a double spend.

- **A dispute is rejected when available funds are insufficient.** If a client deposits 100, withdraws 80, and then disputes the original deposit of 100, the engine would need to move 100 from `available` to `held` — but only 20 remains. Allowing this would produce a negative `available` balance, effectively granting the client credit, which is outside the scope of this system. Instead, the dispute is rejected as a processing error. This is consistent with how withdrawals (as the other transaction where the system could be designed to grant credit) are handled: both are operations that attempt to reduce `available`, and both fail when the balance is too low. In a real-world platform, negative balances and debt recovery would be a separate subsystem.

- **A transaction can only be under one active dispute at a time.** A second dispute on a transaction that is already disputed is ignored. There is no meaningful distinction between "disputed once" and "disputed twice" — the same funds are already held.

- **A frozen account rejects all subsequent transactions.** Once a chargeback freezes an account (`locked = true`), no further deposits, withdrawals, disputes, resolves, or chargebacks are processed for that client. The intended behavior is that the account should be immediately frozen but it is unspecified what happens next; treating it as a hard lock is the safest default and prevents further exposure on a potentially fraudulent account.

- **After a resolve, a transaction may be disputed again.** A resolve returns the transaction to its original, non-disputed state. If a new dispute is later submitted for the same transaction, it is processed normally. This reflects the real-world possibility of a dispute being reopened after initial resolution.

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

### Minimal storage for the transaction log

Deposits must be stored for dispute resolution, but the only field consumed by a dispute (and later resolve/chargeback) is the amount — the client ID is already the outer map key and the transaction ID is the inner map key. Storing the full `Deposit` struct would duplicate both. The transaction log therefore stores only the `Money` amount per entry, minimising per-transaction memory overhead. If future features (e.g., timestamps, dispute windows) require additional metadata, the value type can be promoted to a dedicated struct without changing the `AccountState` API — the storage is fully encapsulated behind its methods.

### Two public APIs: sequential and parallel

The library exposes two entry points: `process()` (sequential, single-threaded) and `process_parallel()` (multi-threaded with client-sharding). Both share the same domain logic — the only difference is the orchestration layer.

The sequential API exists for a reason beyond simplicity: in a **distributed deployment**, a message broker (e.g., Kafka) already partitions the transaction stream by client ID across consumer groups. Each engine instance receives a pre-sharded, ordered stream for its partition — spawning internal worker threads on top of external sharding would add channel and synchronisation overhead for zero benefit. `process()` serves this use case with no threading cost and a lazy `impl Iterator` return type.

`process_parallel()` is designed for **standalone batch processing** without external sharding infrastructure, where the engine itself must shard and parallelise. It requires `Send`-bound callbacks (each callback is moved to a dedicated thread).

| | `process()` | `process_parallel()` |
|---|---|---|
| Use case | Distributed (pre-sharded), small inputs | Standalone batch, large files |
| Threading | None | N workers + 2 callback threads |
| Callback bounds | `FnMut` | `FnMut + Send` |

### No timestamps on transactions or accounts

Timestamps were considered for transactions (for auditing and enabling dispute-window-based eviction) and for accounts (`last_updated`). Both were deferred: the input format provides no event time, so timestamps would reflect processing time only — which is near-identical across a batch run and carries little information. Account-level `last_updated` adds a write on every operation for a field not consumed by the output. In a streaming or real-time system, event-time timestamps become valuable and can be added without changing the processing logic.

## Error Handling

The engine is designed to process large, potentially messy CSV inputs without aborting on the first bad row. Errors are categorised into two kinds:

- **CSV-level errors** — malformed rows that the `csv` crate cannot deserialize (e.g. missing columns, unparseable numbers).
- **Validation errors** — well-formed rows that violate domain rules (e.g. a deposit with a negative or zero amount, an unknown transaction type). These carry structured context: the `client_id`, `tx_id`, and a human-readable message.
- **Processing errors** — valid transactions that conflict with the current account state (e.g. a withdrawal exceeding the available balance, a dispute on an already-disputed transaction, any operation on a frozen account). These also carry `client_id`, `tx_id`, and a descriptive message.

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

### Test Strategy

The test suite is organised into three layers:

**Unit tests** (`src/*/tests.rs`) cover individual modules — parsing, output serialization, and domain logic — in isolation.

**Targeted integration tests** (`tests/deposit/`, `tests/from_file/`) exercise specific transaction types through the public `process` API or the compiled binary, using hand-crafted inputs and expected outputs.

**Scenario-based property tests** (`tests/scenarios/`) provide broad, randomised coverage of the engine's overall behaviour. The approach works as follows:

1. **Scenario shapes** are defined in a catalog (`tests/scenarios/catalog.rs`). Each shape describes a fixed sequence of transaction types (e.g. "single deposit", "two deposits") and a formula for the expected outcome — the final account state and which transaction IDs should succeed or error. The formula is trivial arithmetic, not a reimplementation of the engine logic.

2. **`proptest`** generates random parameters for each test run: which shapes to combine, the monetary amounts, and a seed for interleaving. Each shape is assigned a unique client ID and non-overlapping transaction ID range, then built into a concrete `Scenario` with real CSV rows and expected results.

3. **Order-preserving interleaving** combines multiple scenarios into a single CSV input. A shuffled schedule (seeded by `proptest` for reproducibility) determines the order in which rows from different clients appear, while preserving each client's internal transaction chronology. This tests client isolation — one client's transactions must never affect another's state, regardless of how they are interleaved.

4. **Assertion** runs the interleaved CSV through `process`, collecting accounts, successes, and errors into per-client hash maps, then checks each scenario's expectations: account state equality, correct success transaction IDs, and correct error transaction IDs.

Adding a new transaction type requires only defining a new scenario shape in the catalog and registering it in `all_shapes()`. The proptest driver, interleaving logic, and assertion infrastructure remain unchanged.

The test infrastructure itself (interleaving, schedule generation, result collection) is covered by its own dedicated tests to ensure the harness is trustworthy.

## CI

Every pull request against `main` runs a GitHub Actions pipeline that enforces:

- **Formatting** — `cargo fmt --all --check` ensures consistent style.
- **Linting** — `cargo clippy --all-targets -- -D warnings` catches common mistakes and enforces idiomatic Rust.
- **Dependency audit** — `cargo deny check advisories` flags known vulnerabilities in dependencies.
- **Tests** — `cargo nextest run` runs the full test suite.
- **Coverage** — `cargo llvm-cov nextest --fail-under-lines 90` enforces a minimum of 90 % line coverage.

The pipeline definition lives in `.github/workflows/ci.yml`.

## Performance

Benchmarks show that the **sequential `process()` API is ~3.5× faster** than the parallel variant for the chosen workload (~176K transactions in ~180 ms vs ~636 ms). The per-transaction work — a HashMap lookup and decimal arithmetic — is so lightweight that channel synchronisation overhead dominates any parallelism benefit. The binary therefore uses single-threaded processing by default.

Full methodology, reproduction instructions, and analysis are in [PERFORMANCE.md](./PERFORMANCE.md).

## Future Work

- **Write-ahead log, checkpointing & recovery:** Persist transactions to a WAL before processing, with periodic snapshots of account state. On crash, replay the WAL from the last checkpoint to restore consistent state without reprocessing the full input.

- **Transaction timestamps & dispute windows:** In a streaming system, transactions could carry event-time timestamps, enabling eviction of old transactions that are past their dispute window — reducing memory usage in long-running deployments.

- **Batched parallel dispatch:** The current parallel implementation sends one transaction per channel message. Batching multiple transactions into a single message would amortise synchronisation cost and could make the parallel mode competitive with sequential processing (see [PERFORMANCE.md](./PERFORMANCE.md)).

- **`u64` money representation:** Replace `rust_decimal::Decimal` with fixed-point `u64` arithmetic (e.g., storing ten-thousandths of a unit). This would reduce per-value memory, eliminate heap allocation during parsing, and improve cache locality — at the cost of slightly more verbose formatting and the need for overflow checks.

- **Transaction deduplication:** Reject transactions that reuse an existing transaction ID, providing idempotency guarantees for at-least-once delivery systems.

- **Out-of-order transaction handling:** The engine currently assumes that transactions arrive in chronological order — a simplification that is unlikely to hold in distributed or high-throughput environments. Supporting out-of-order delivery would require buffering, sequencing (e.g., via event-time timestamps or sequence numbers), and potentially reworking the dispute/resolve/chargeback state machine to handle "future" references gracefully. This would be a substantial change to the processing model.

---
**AI Usage Declaration:** This project utilized generative AI tools (Cursor) for architectural brainstorming and boilerplate generation. All code and logic were reviewed, tested, and validated by the author to ensure correctness and security. For a detailed log, see [AI_USAGE.md](./AI_USAGE.md).