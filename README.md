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

mVMC input files are generated in the `output/` directory.

## Output Files

| File | Description |
|------|-------------|
| `Trans.def` | One-body transfer integrals |
| `InterAll.def` | Two-body interaction terms |
| `modpara.def` | Simulation parameters |
| `locspn.def` | Local spin configuration |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |

## Documentation

- [Input syntax reference](docs/) (coming soon)

## License

TBD
