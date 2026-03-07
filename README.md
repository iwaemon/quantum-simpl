# quantum-simpl

[日本語](README.ja.md)

A Hamiltonian symbolic preprocessor that generates [mVMC](https://github.com/issp-center-dev/mVMC) input files from a simple model definition.

## What is this?

[mVMC](https://github.com/issp-center-dev/mVMC) is a variational Monte Carlo solver for quantum lattice models. It requires several structured input files (`Trans.def`, `InterAll.def`, etc.) to define the Hamiltonian — writing these by hand is tedious and error-prone for large models.

quantum-simpl automates this. You write a short model definition, and it:

1. **Expands** — unrolls sum loops, expands Hermitian conjugates, desugars `n(i,s)`
2. **Normal orders** — applies fermion anticommutation and spin commutation relations
3. **Combines** — deduplicates identical operator strings, sums coefficients
4. **Filters** — removes terms that break Sz conservation
5. **Outputs** — writes all mVMC input files

## Quick Start

### Install

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
# Binary at target/release/quantum-simpl
```

### Run

Create an input file `hubbard.def`:

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

Run quantum-simpl:

```bash
quantum-simpl hubbard.def -o output/
```

mVMC input files are generated in the `output/` directory. Input files can use any extension (`.qsl`, `.def`, etc.).

### Heisenberg Model Example

quantum-simpl also supports spin operators. Create `heisenberg.qsl`:

```
lattice 1d sites=10 pbc=true

sum i=0..10:
  J * Sp(i) * Sm(i+1)
  J * Sm(i) * Sp(i+1)
  J * Sz(i) * Sz(i+1)

params:
  J = 1.0
```

```bash
quantum-simpl heisenberg.qsl -o output/
```

### Note on Open Boundary Conditions

When using `pbc=false`, terms that reference sites outside the lattice range are silently dropped. Make sure your sum ranges are compatible with the lattice size. For example, with `sites=10 pbc=false`, use `sum i=0..9` for nearest-neighbor hopping to avoid referencing site 10.

## Supported Operators

| Operator | Syntax | Description |
|----------|--------|-------------|
| Creation | `c†(i,spin)` | Fermion creation operator |
| Annihilation | `c(i,spin)` | Fermion annihilation operator |
| Number | `n(i,spin)` | Number operator (sugar for `c†(i,s) c(i,s)`) |
| Spin+ | `Sp(i)` | Spin raising operator |
| Spin- | `Sm(i)` | Spin lowering operator |
| Spin-z | `Sz(i)` | Spin z-component operator |

Spin values: `up`, `down`. Index expressions: `i`, `i+1`, `i-1`, or literal integers.

## Output Files

| File | Description |
|------|-------------|
| `namelist.def` | Master index of input files |
| `modpara.def` | Simulation parameters |
| `locspn.def` | Local spin configuration |
| `trans.def` | One-body transfer integrals |
| `interall.def` | Two-body interaction terms |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |
| `qptransidx.def` | Quantum number projection parameters |

## Documentation

- [Input syntax reference](docs/) (coming soon)

## License

TBD
