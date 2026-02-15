# AI Usage Documentation

## General Approach

This project was developed with AI assistance (Claude, via Cursor IDE). The AI was used primarily as a **sparring partner** in *Ask mode* to brainstorm concepts, validate design decisions, and discuss trade-offs. Implementation was done in *Agent mode* for individual, precisely scoped pieces of work — always directed by explicit instructions.

All architectural decisions, domain modeling, and error-handling strategies were made by me; the AI served as an accelerator, not a designer.

## Scope

Smaller interactions (e.g., quick fixes, minor refactors), the use of Agent mode to adjust documentation files, and Agent-generated test cases based on scenarios I described are not documented here.

## Interaction Log

### 1 — README Structure

- **Mode:** Ask
- **Context:** Discussed what sections the project README should contain and how to structure it.
- **Outcome:** Defined README sections covering overview, design decisions, assumptions, error handling, and usage instructions.

### 2 — From-File Test skeletton

- **Mode:** Ask
- **Context:** Draft for the from-file test, based on a given directory structure.
- **Outcome:** Draft slightly modified and used as the "from-file" test for the case of "two_deposits".

### 3 — Tokio: Use or Not?

- **Mode:** Ask
- **Context:** Discussed whether to include `tokio` from the start, given the planned architecture of one worker per thread with no shared memory between threads.
- **Outcome:** Decided against using `tokio` for now. The workload is CPU-bound and synchronous — workers receive transactions and update in-memory balances with no I/O to await. `tokio` compile-time cost without benefit here. `tokio` will be introduced when the system boundary changes (e.g., network ingestion, async I/O in the pipeline), at which point the pure, synchronous domain logic can slot cleanly into it.

### 4 — Domain Types Review

- **Mode:** Ask
- **Context:** Reviewed initial draft of core domain types (`Transaction`, `ClientId`, `TxId`, `AccountState`). Discussed the `Decimal` vs `u64` trade-off for monetary values, whether `total` should be stored or computed, and whether to add timestamps to transactions and accounts.
- **Outcome:** Removed `total` from `AccountState` (computed on the fly instead — impossible to drift out of sync). Deferred timestamps (no event time in input, processing time is meaningless in batch mode). Both decisions documented in README under Design Decisions and Future Work.

### 5 — Input Parsing Design & Implementation

- **Mode:** Ask + Agent
- **Context:** Designed the input parsing layer: how to split I/O from parsing, how the `csv` crate integrates, test structure, and error-handling strategy. Also reviewed the implemented parsing logic and tests.
- **Outcome:** Generated a first draft for the parsing logic and the parsing unit tests. Decided against `thiserror` for now — `anyhow` suffices until there are multiple distinct error variants callers need to branch on.

### 6 — Parameterized Tests with `rstest`

- **Mode:** Ask
- **Context:** Explored options for parameterized/property-based testing of the parsing logic (`proptest` vs `rstest`). Discussed how to structure a single cross-product test that encodes the validation rules and scales as new transaction types are added.
- **Outcome:** Chose `rstest` with `#[values]` over `proptest` — the input space is a small, well-defined partition (amount: positive / zero / negative / missing × transaction type), not a fuzzing problem. Generated a draft for a single parameterized test with inline validation logic.

### 7 — Output Module Design & Tests

- **Mode:** Ask + Agent
- **Context:** Designed the output layer for converting domain types to serializable output. Discussed the split between lib and main, return type (Vec vs iterator), and whether to sort output.
- **Outcome:** The lib output module converts `HashMap<ClientId, AccountState>` into an iterator of a `Serialize`-able DTO (`AccountRecord`), keeping the lib format-agnostic — `main` picks the serialization format. No sorting in production code since row ordering is not required and the integration tests already normalize order. Generated a draft for the output unit tests.

### 8 — Error Type Design with `thiserror`

- **Mode:** Ask + Agent
- **Context:** Discussed how to structure the crate's error type — whether to have detailed enum variants per validation case or a flatter design, and how to include transaction context in errors.
- **Outcome:** Introduced `thiserror` with a flat `Error` enum: `Csv` (wrapping `csv::Error` via `#[from]`) and `Validation` (a struct variant with `client_id`, `tx_id`, and `message`). Detailed per-case variants were deferred since callers don't need to branch on specific validation failures. Agent refactored code from `anyhow` to `thiserror` errors.

### 9 — Public API & Caller-Defined Error Handling

- **Mode:** Ask + Agent
- **Context:** Designed the public `process` entry point and its error-handling API. Discussed how to give callers full control over what happens with erroneous transactions, and how this design extends to a future multi-threaded architecture where workers send errors through channels.
- **Outcome:** `process` accepts an `on_error: impl FnMut(Error)` callback — the library reports each error and skips the transaction, while the caller defines the policy (log, collect, abort, etc.). The binary passes a simple logging function. Documented the approach in the README under both Error Handling and Design Decisions. Agent annotated the `process` function with a doc comment covering usage, error semantics, and an example.

### 10 — Success Callback & `TransactionRecord` DTO

- **Mode:** Ask + Agent
- **Context:** Discussed adding an `on_success` callback to give callers visibility into successfully processed transactions (for logging, metrics, publishing). This raised the question of what type to expose — the internal `Transaction` domain type or a public DTO. Initially made `Transaction` public, but decided against it to keep domain types encapsulated.
- **Outcome:** Introduced `TransactionRecord` as a public DTO in the output module (mirroring the `AccountRecord` pattern), with `Display`, `Copy`, and `PartialEq`. The engine converts domain transactions to `TransactionRecord` before calling `on_success`. The binary logs each success at `info` level. Also fixed a bug where `tracing`'s default `fmt::layer()` was writing to stdout instead of stderr, which broke the E2E test — resolved by adding `.with_writer(std::io::stderr)`.

### 11 — Integration Testing Strategy

- **Mode:** Ask
- **Context:** Brainstormed how to test the overall engine behavior across transaction types without reimplementing the processing logic inside the tests. Discussed parametrized tests, property-based testing, and the constraint that per-client transaction order must be preserved (transactions are chronological in the input).
- **Outcome:** Settled on a scenario-based approach: each scenario is a self-contained per-client story (a fixed "shape" of transaction types with parametrized amounts) paired with a known expected outcome — both the final account state and which transactions should succeed or error. Multiple scenarios with distinct client IDs are combined via an order-preserving random interleave (a shuffled schedule that preserves each client's internal chronology). Amounts and interleaving are generated by `proptest`. This avoids reimplementing logic (expected values are trivial arithmetic per shape), tests client isolation and ordering correctness, and scales naturally as new transaction types are added — just define new scenario shapes.

### 12 — Scenario Test Infrastructure Implementation

- **Mode:** Agent (supervised)
- **Context:** Implemented the full `tests/scenarios/` module based on the strategy agreed in entry 11. This included the `Scenario` type, `ScenarioShape` trait, `interleave` function (order-preserving schedule-driven CSV generation), `run_process` (process wrapper collecting results into per-client hash maps), `assert_scenarios` (per-client equality checks), the `build_schedule` function (Fisher-Yates shuffle with a seeded LCG), and the `proptest!` driver that wires everything together.
- **Outcome:** Agent generated the infrastructure code and tests for the infrastructure (interleave correctness, panic on over/underrun, schedule determinism, schedule count correctness, full roundtrip). All code was reviewed and adjusted during the process — e.g., switching `ProcessResult` from flat vectors to per-client hash maps for simpler assertions, removing stub helpers in favour of real catalog scenarios, and clarifying the tx ID offset convention.

### 13 — Test Coverage Gap Analysis & Additional Tests

- **Mode:** Ask + Agent
- **Context:** Before starting on the core logic, reviewed existing test coverage to identify gaps. Identified five potential areas: empty input, exact-balance withdrawal boundary, invalid withdrawal amounts (zero/negative), multi-client isolation in targeted tests, and withdrawal on a never-seen client.
- **Outcome:** Added three items based on my prioritisation: an empty-input edge-case test in the integration root, an `ExactBalanceWithdrawal` scenario shape in the catalog (deposit X, withdraw X, balance = 0), and two targeted tests for zero- and negative-amount withdrawals. Multi-client isolation was already covered by the scenario interleaving; the never-seen-client case was deemed unnecessary given existing overdraft coverage.

### 14 — Dispute, Resolve & Chargeback Semantics

- **Mode:** Ask
- **Context:** Discussed the intended behavior of the three remaining transaction types (dispute, resolve, chargeback) based on their intended behavior. With a general description of a dispute, several edge cases are ambiguous — particularly whether withdrawals can be disputed, what happens when disputed funds have already been spent, and what a frozen account means for subsequent transactions. Worked through each case by reasoning from real-world financial semantics and consistency with the existing engine rules.
- **Outcome:** Established and documented seven assumptions in the README: only deposits can be disputed (withdrawals are between the client and the destination); a dispute is rejected when `available` is insufficient (consistent with the no-negative-balance rule for withdrawals); a frozen account rejects all subsequent transactions (safest default given the ambiguity); one active dispute per transaction; resolved transactions may be re-disputed; and zero-amount withdrawals are rejected. The "chargebacks are terminal" point was initially drafted but removed as redundant — it is already implied by the frozen-account rule, which explicitly lists chargebacks among rejected types.

### 15 — Dispute Test Cases

- **Mode:** Ask + Agent
- **Context:** Before implementing dispute logic, defined the full set of dispute test cases — both as targeted integration tests (`tests/dispute/mod.rs`) and as scenario shapes for the proptest catalog. Discussed which cases to cover, whether error-only cases belong in the scenario catalog (decided yes — scenarios document behavior and gain value from interleaving), and parameter encoding for each shape.
- **Outcome:** Six dispute cases implemented as both targeted tests and scenario shapes: happy-path deposit-then-dispute, dispute on a nonexistent tx, dispute on a withdrawal, dispute with insufficient available funds, double dispute on the same tx, and two-deposits-dispute-first (verifying only the targeted deposit is affected). None of the production code was touched — tests are written ahead of implementation, matching the approach used for withdrawals.

### 16 — Resolve Test Cases

- **Mode:** Ask + Agent
- **Context:** Before implementing resolve logic, defined the full set of resolve test cases — targeted integration tests (`tests/resolve/mod.rs`) and scenario shapes. Discussed edge cases such as resolving a non-existent transaction, resolving an undisputed deposit, double resolve, and re-dispute after resolve.
- **Outcome:** Eight resolve cases implemented as both targeted tests and scenario shapes: happy-path dispute-then-resolve, resolve on a nonexistent tx, resolve on an undisputed deposit, double resolve, re-dispute after resolve, and multi-deposit scenarios verifying that only the targeted deposit is affected by the resolve. Tests written ahead of implementation.

### 17 — Chargeback Test Cases

- **Mode:** Ask + Agent
- **Context:** Before implementing chargeback logic, defined the full set of chargeback test cases — targeted integration tests (`tests/chargeback/mod.rs`) and scenario shapes. Discussed edge cases including chargebacks on non-existent transactions, undisputed deposits, double chargebacks, operations on frozen accounts, and chargeback on a different previously-disputed transaction after an account is already frozen.
- **Outcome:** Nine chargeback cases implemented as both targeted tests and scenario shapes: happy-path dispute-then-chargeback, chargeback on a nonexistent tx, chargeback on an undisputed deposit, double chargeback, deposit/withdrawal/dispute/resolve/chargeback on a frozen account, and chargeback on a different disputed tx after freeze. Tests written ahead of implementation.
