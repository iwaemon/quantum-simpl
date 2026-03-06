# quantum-simpl: Hamiltonian Symbolic Preprocessor

## Overview

A CLI tool in Rust that takes a model definition (custom DSL) and produces mVMC-ready input files. It preprocesses Hamiltonians by expanding, normal-ordering, combining like terms, and applying spin symmetry reduction.

## Target Models

- Extended Hubbard models (fermionic: c†, c)
- Heisenberg-type models (spin: S+, S-, Sz)
- Scale: 100k+ operator terms

## Input DSL

```
lattice 1d sites=10 pbc=true

sum i=0..10:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
```

Features:
- `lattice` declaration with geometry, size, boundary conditions
- `sum` blocks with index ranges
- `h.c.` shorthand for Hermitian conjugate
- `n(i,s)` sugar for `c†(i,s) c(i,s)`
- Spin operators: `Sp(i)`, `Sm(i)`, `Sz(i)`
- `params` block for numeric values

## Architecture: Flat Term Table

### Data Model

```rust
enum Spin { Up, Down }

enum Op {
    FermionCreate(usize, Spin),      // c†(site, spin)
    FermionAnnihilate(usize, Spin),  // c(site, spin)
    SpinPlus(usize),                 // S+(site)
    SpinMinus(usize),                // S-(site)
    SpinZ(usize),                    // Sz(site)
}

struct Term {
    coeff: f64,
    ops: SmallVec<[Op; 4]>,  // stack-allocated for ≤4 operators
}

struct Hamiltonian {
    terms: Vec<Term>,
    num_sites: usize,
}
```

### Processing Pipeline

```
Input DSL → [Expand] → [Normal Order] → [Combine] → [Symmetry] → mVMC output
```

1. **Expand** — unroll `sum` loops, expand `h.c.`, desugar `n(i,s)`, substitute parameters
2. **Normal Order** — move c† left of c (anticommutation), apply spin commutation relations
3. **Combine** — hash-based deduplication of identical operator strings, sum coefficients, drop zeros
4. **Symmetry** — Sz conservation: filter terms by quantum number sector

### Output

- **Hamiltonian files** (generated from DSL): `Trans.def`, `InterAll.def`, etc.
- **Other mVMC files** (default templates): `modpara.def`, `gutzwilleridx.def`, etc.

## Project Structure

```
quantum-simpl/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point
│   ├── parser/
│   │   ├── mod.rs       # DSL parser
│   │   └── ast.rs       # Parse tree
│   ├── core/
│   │   ├── mod.rs
│   │   ├── op.rs        # Op, Term, Hamiltonian
│   │   ├── expand.rs    # Step 1: expansion
│   │   ├── normal.rs    # Step 2: normal ordering
│   │   ├── combine.rs   # Step 3: like-term combining
│   │   └── symmetry.rs  # Step 4: Sz symmetry reduction
│   └── output/
│       ├── mod.rs
│       └── mvmc.rs      # mVMC format writer
└── tests/
    └── integration/     # Integration tests with known models
```

## Dependencies

- `clap` — CLI argument parsing
- `smallvec` — stack-allocated small vectors
- `rustc-hash` — FxHashMap for fast hashing
- `rayon` — parallel processing for large Hamiltonians

## Design Decisions

- **Flat Term Table over Expression Tree**: better cache locality and simpler parallelization at 100k+ terms
- **SmallVec<[Op; 4]>**: most terms in Hubbard/Heisenberg models have ≤4 operators, avoiding heap allocation
- **f64 coefficients**: no symbolic coefficient math needed
- **FxHashMap for combining**: fast non-cryptographic hashing for like-term detection
