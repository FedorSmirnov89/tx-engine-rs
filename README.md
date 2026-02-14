# tx-engine-rs

A high-performance transaction processing engine in Rust. Ingests a stream of financial transactions from CSV, processes them in memory, and outputs the final state of all client accounts to STDOUT.

## Usage

TODO

## Assumptions

TODO

## Design Decisions

TODO

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

TODO


---
**AI Usage Declaration:** This project utilized generative AI tools (Cursor) for architectural brainstorming and boilerplate generation. All code and logic were reviewed, tested, and validated by the author to ensure correctness and security. For a detailed log, see [AI_USAGE.md](./AI_USAGE.md).