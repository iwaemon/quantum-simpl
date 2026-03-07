# Contributing

## Build Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (binary at target/release/quantum-simpl)
cargo test                           # Run all tests
cargo test test_pipeline             # Run a single integration test
cargo test unit_parser               # Run a single unit test file
cargo test --test test_hubbard       # Run integration test by name
```

## Test Structure

Tests are organized into two categories:

### Unit tests (`tests/unit_*.rs`)

Each pipeline stage has a corresponding unit test file:

| Test file | Covers |
|-----------|--------|
| `unit_parser` | DSL parsing |
| `unit_expand` | Sum loop expansion, h.c., parameter substitution |
| `unit_normal` | Fermion anticommutation, normal ordering |
| `unit_combine` | Term deduplication, coefficient summing |
| `unit_symmetry` | Sz conservation filtering |
| `unit_op` | Operator type construction and properties |
| `unit_mvmc` | mVMC output file formatting |
| `unit_transform` | YS transform and spin-to-fermion conversion |
| `unit_classify` | Term classification (one-body, coulomb-intra, two-body) |
| `unit_green` | Green function reordering |
| `unit_correlation` | Correlation summary output |

### Integration tests (`tests/integration/`)

End-to-end tests that run the full pipeline:

- `test_pipeline` -- basic pipeline flow
- `test_hubbard` -- Hubbard model end-to-end
- `test_heisenberg` -- Heisenberg model end-to-end
- `test_mvmc_output` -- verifies generated mVMC files
- `test_ys_transform` -- Yokoyama-Shiba transform pipeline
- `test_ys_validation` -- YS edge cases and validation
- `test_correlation` -- correlation function pipeline

## Running Specific Tests

Run a single unit test file:

```bash
cargo test unit_parser
```

Run a single integration test:

```bash
cargo test --test test_hubbard
```

Run tests matching a pattern:

```bash
cargo test expand    # runs all tests with "expand" in the name
```

## Code Organization

- **`src/parser/`** -- DSL parser (hand-written, line-based)
- **`src/core/`** -- Pipeline transformation stages
- **`src/output/`** -- Output formatters (mVMC files, correlation summary)
- **`src/main.rs`** -- CLI entry point, pipeline orchestration

See the [Architecture](architecture.md) page for detailed module descriptions.
