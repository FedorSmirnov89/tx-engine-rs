# tx-engine-rs

A high-performance transaction processing engine in Rust. Ingests a stream of financial transactions from CSV, processes them in memory, and outputs the final state of all client accounts to STDOUT.

## Usage

TODO

## Assumptions

- **Zero-amount deposits are rejected.** A deposit of `0.0` has no effect on account balances but would still consume memory when stored for dispute resolution. These are treated as invalid input.

## Design Decisions

### No async runtime (yet)

The processing workload is CPU-bound and synchronous — workers receive transactions and update in-memory balances with nothing to `await`. Adding an async runtime (`tokio`) would introduce compile-time overhead without benefit. The domain logic is kept purely synchronous, making it straightforward to integrate with an async runtime later.

### Money representation: `Decimal` over `u64`

The two main candidates for representing monetary values are `u64` (storing the smallest unit, e.g., ten-thousandths) and `rust_decimal::Decimal`. `u64` is more compact and inherently non-negative — which fits this domain, since balances should never go negative by design. However, `Decimal` offers easier parsing from the CSV input format and simpler formatting on output, reducing boilerplate at this stage. Since all monetary fields are accessed through a type alias, switching to `u64` later is a low-cost optimization if needed.

### No timestamps on transactions or accounts

Timestamps were considered for transactions (for auditing and enabling dispute-window-based eviction) and for accounts (`last_updated`). Both were deferred: the input format provides no event time, so timestamps would reflect processing time only — which is near-identical across a batch run and carries little information. Account-level `last_updated` adds a write on every operation for a field not consumed by the output. In a streaming or real-time system, event-time timestamps become valuable and can be added without changing the processing logic.

## Error Handling

TODO

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

## Future Work

- **Transaction timestamps & dispute windows:** In a streaming system, transactions could carry event-time timestamps, enabling eviction of old transactions that are past their dispute window — reducing memory usage in long-running deployments.


---
**AI Usage Declaration:** This project utilized generative AI tools (Cursor) for architectural brainstorming and boilerplate generation. All code and logic were reviewed, tested, and validated by the author to ensure correctness and security. For a detailed log, see [AI_USAGE.md](./AI_USAGE.md).