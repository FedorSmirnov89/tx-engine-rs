# AI Usage Documentation

## General Approach

This project was developed with AI assistance (Claude, via Cursor IDE). The AI was used primarily as a **sparring partner** in *Ask mode* to brainstorm concepts, validate design decisions, and discuss trade-offs. Implementation was done in *Agent mode* for individual, precisely scoped pieces of work — always directed by explicit instructions.

All architectural decisions, domain modeling, and error-handling strategies were made by me; the AI served as an accelerator, not a designer.

## Scope

Smaller interactions (e.g., quick fixes, minor refactors) and the use of Agent mode to adjust documentation files are not documented here.

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
- **Outcome:** The lib output module converts `HashMap<ClientId, AccountState>` into an iterator of a `Serialize`-able DTO (`AccountRecord`), keeping the lib format-agnostic — `main` picks the serialization format. No sorting in production code since the spec doesn't require it and the integration tests already normalize order. Generated a draft for the output unit tests.

### 8 — Error Type Design with `thiserror`

- **Mode:** Ask + Agent
- **Context:** Discussed how to structure the crate's error type — whether to have detailed enum variants per validation case or a flatter design, and how to include transaction context in errors.
- **Outcome:** Introduced `thiserror` with a flat `Error` enum: `Csv` (wrapping `csv::Error` via `#[from]`) and `Validation` (a struct variant with `client_id`, `tx_id`, and `message`). Detailed per-case variants were deferred since callers don't need to branch on specific validation failures. Agent refactored code from `anyhow` to `thiserror` errors.