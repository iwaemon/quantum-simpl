# Installation

## Prerequisites

- **Rust toolchain** (cargo, rustc). Install via [rustup](https://rustup.rs/) if not already present.

## Install

Install directly with cargo:

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

The release binary is located at `target/release/quantum-simpl`.

## Verify

```bash
quantum-simpl --help
```

This should print the available command-line options.
