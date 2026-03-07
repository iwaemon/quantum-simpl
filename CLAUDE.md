# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

quantum-simpl is a Rust CLI tool that generates [mVMC](https://github.com/issp-center-dev/mVMC) input files from a simple Hamiltonian DSL. It targets Hubbard and Heisenberg quantum lattice models.

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (binary at target/release/quantum-simpl)
cargo test                           # Run all tests
cargo test test_pipeline             # Run a single integration test
cargo test unit_parser               # Run a single unit test file
cargo test --test test_hubbard       # Run integration test by name
```

## Architecture

### Two pipelines

The CLI (`src/main.rs`) has two distinct pipelines selected by flags:

**Standard pipeline** (default, processes `<INPUT>`):
```
Parse → Expand → Spin→Fermion → [YS Transform] → Normal Order → Combine → Sz Filter → [Classify] → mVMC Output
```

**Correlation pipeline** (`--correlation <FILE>`):
```
Parse → Expand → Spin→Fermion → Normal Order → Combine → Green Reorder → cisajs/cisajscktaltdc Output
```

The YS transform path also triggers term classification (one-body / coulomb-intra / two-body) and writes a `coulombintra.def` when applicable.

### Module layout

- **`src/parser/`** — Hand-written line-based DSL parser. `ast.rs` defines the parse tree types (`ModelDef`, `SumBlock`, `Expression`, `OpExpr`, etc.). `mod.rs` contains the parser logic.
- **`src/core/`** — Transformation pipeline stages:
  - `op.rs` — Core data types: `Op` (enum of fermion/spin operators), `Term` (coeff + SmallVec of ops), `Hamiltonian`
  - `expand.rs` — Unrolls sum loops, expands h.c., desugars `n(i,s)` → `c†c`, substitutes params
  - `normal.rs` — Applies fermion anticommutation to achieve normal ordering (c† before c)
  - `combine.rs` — Hash-based deduplication of identical operator strings, sums coefficients
  - `symmetry.rs` — Filters terms that violate Sz conservation
  - `transform.rs` — Substitution rules: particle-hole (YS) transform, spin-to-fermion conversion (`Sp/Sm/Sz` → `c†c`)
  - `classify.rs` — Splits terms into constants, one-body, coulomb-intra, and two-body categories (used by YS path)
  - `green.rs` — Reorders 4-operator terms into Green's function form (`c†cc†c`) with anticommutation corrections
- **`src/output/`**:
  - `mvmc.rs` — Writes mVMC-format `.def` files (namelist, modpara, trans, interall, cisajs, cisajscktaltdc, etc.)
  - `correlation.rs` — Human-readable `correlation_summary.txt` formatter

### Key design choices

- **Flat Term Table**: Terms are stored as `Vec<Term>` rather than an expression tree — better cache locality for 100k+ terms
- **`SmallVec<[Op; 4]>`**: Most Hubbard/Heisenberg terms have ≤4 operators, so ops are stack-allocated
- **`FxHashMap`** (from `rustc-hash`): Used in combine step for fast non-cryptographic hashing

### Tests

- `tests/unit_*.rs` — Unit tests for individual pipeline stages (parser, expand, normal, combine, symmetry, op, mvmc, transform, classify, green, correlation)
- `tests/integration/` — End-to-end tests: `test_pipeline`, `test_hubbard`, `test_heisenberg`, `test_mvmc_output`, `test_ys_transform`, `test_ys_validation`, `test_correlation`

## DSL Syntax

```
lattice 1d sites=N pbc=true|false

sum var=start..end:
  coeff * c†(index,spin) c(index,spin) + h.c.
  coeff * n(index,spin) n(index,spin)

params:
  name = value
```

Operators: `c†(i,s)`, `c(i,s)`, `n(i,s)` (sugar for c†c), `Sp(i)`, `Sm(i)`, `Sz(i)`. Spin values: `up`, `down`. Index expressions support `var`, `var+offset`, `var-offset`, or literal integers.

## CLI Usage

```bash
quantum-simpl input.def -o output/           # Standard pipeline
quantum-simpl input.def -o output/ --ys-transform  # With Yokoyama-Shiba transform
quantum-simpl --correlation corr.def -o output/    # Correlation pipeline only
quantum-simpl input.def --correlation corr.def -o output/  # Both pipelines
```
