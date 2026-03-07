# Architecture

## Two Pipelines

The CLI (`src/main.rs`) has two distinct pipelines selected by flags.

### Standard pipeline (default)

Processes the `<INPUT>` file and generates mVMC Hamiltonian input files:

```
Parse -> Expand -> Spin->Fermion -> [YS Transform] -> Normal Order -> Combine -> Sz Filter -> [Classify] -> mVMC Output
```

Stages in brackets are optional. The YS transform path also triggers term classification (one-body / coulomb-intra / two-body) and writes `coulombintra.def` when applicable.

### Correlation pipeline (`--correlation <FILE>`)

Processes a correlation definition file and generates measurement input files:

```
Parse -> Expand -> Spin->Fermion -> Normal Order -> Combine -> Green Reorder -> cisajs/cisajscktaltdc Output
```

The key difference is the Green Reorder stage, which converts 4-operator terms from normal-ordered form (`c+c+cc`) into Green's function form (`c+c c+c`) with anticommutation corrections.

## Module Layout

### `src/parser/`

Hand-written line-based DSL parser.

- `ast.rs` -- parse tree types: `ModelDef`, `SumBlock`, `Expression`, `OpExpr`, etc.
- `mod.rs` -- parser logic

### `src/core/`

Transformation pipeline stages:

| Module | Purpose |
|--------|---------|
| `op.rs` | Core data types: `Op` (enum of fermion/spin operators), `Term` (coeff + SmallVec of ops), `Hamiltonian` |
| `expand.rs` | Unrolls sum loops, expands h.c., desugars `n(i,s)` to `c+c`, substitutes params |
| `normal.rs` | Applies fermion anticommutation to achieve normal ordering (c+ before c) |
| `combine.rs` | Hash-based deduplication of identical operator strings, sums coefficients |
| `symmetry.rs` | Filters terms that violate Sz conservation |
| `transform.rs` | Substitution rules: particle-hole (YS) transform, spin-to-fermion conversion (`Sp/Sm/Sz` to `c+c`) |
| `classify.rs` | Splits terms into constants, one-body, coulomb-intra, and two-body categories (used by YS path) |
| `green.rs` | Reorders 4-operator terms into Green's function form (`c+c c+c`) with anticommutation corrections |

### `src/output/`

| Module | Purpose |
|--------|---------|
| `mvmc.rs` | Writes mVMC-format `.def` files (namelist, modpara, trans, interall, etc.) |
| `correlation.rs` | Human-readable `correlation_summary.txt` formatter |

## Key Design Choices

### Flat Term Table

Terms are stored as `Vec<Term>` rather than an expression tree. This provides better cache locality when processing 100k+ terms, which is common for large lattice models.

### SmallVec<[Op; 4]>

Most Hubbard and Heisenberg terms have 4 or fewer operators. Using `SmallVec<[Op; 4]>` from the `smallvec` crate keeps operators stack-allocated for the common case, avoiding heap allocations.

### FxHashMap

The combine step uses `FxHashMap` from the `rustc-hash` crate for fast non-cryptographic hashing. This is significantly faster than the standard `HashMap` for the integer-heavy keys used in operator string deduplication.

## Test Structure

### Unit tests

Located in `tests/unit_*.rs`, covering individual pipeline stages:

- `unit_parser` -- DSL parsing
- `unit_expand` -- sum expansion and h.c.
- `unit_normal` -- normal ordering
- `unit_combine` -- term deduplication
- `unit_symmetry` -- Sz filtering
- `unit_op` -- operator types
- `unit_mvmc` -- mVMC output formatting
- `unit_transform` -- YS and spin-to-fermion transforms
- `unit_classify` -- term classification
- `unit_green` -- Green reordering
- `unit_correlation` -- correlation output

### Integration tests

Located in `tests/integration/`, testing complete pipelines end-to-end:

- `test_pipeline` -- basic pipeline flow
- `test_hubbard` -- Hubbard model
- `test_heisenberg` -- Heisenberg model
- `test_mvmc_output` -- mVMC file generation
- `test_ys_transform` -- Yokoyama-Shiba transform
- `test_ys_validation` -- YS validation edge cases
- `test_correlation` -- correlation pipeline
