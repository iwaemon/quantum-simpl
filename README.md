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

sum i=0..9:
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

**CLI options:**

- `quantum-simpl <INPUT> -o <DIR>` — Standard pipeline (Hamiltonian → mVMC files).
- `quantum-simpl --correlation <FILE> -o <DIR>` — Correlation pipeline only (generates measurement files).
- `quantum-simpl <INPUT> --correlation <FILE> -o <DIR>` — Run both pipelines.
- `--ys-transform` — Apply Yokoyama–Shiba (particle-hole for down-spin) in the standard pipeline (see below).

### Heisenberg Model Example

quantum-simpl also supports spin operators. Create `heisenberg.qsl`:

```
lattice 1d sites=10 pbc=true

sum i=0..9:
  J * S(i) . S(i+1)

params:
  J = 1.0
```

```bash
quantum-simpl heisenberg.qsl -o output/
```

### Note on Open Boundary Conditions

When using `pbc=false`, terms that reference sites outside the lattice range are dropped with a warning. Make sure your sum ranges are compatible with the lattice size. For example, with `sites=10 pbc=false`, use `sum i=0..8` for nearest-neighbor hopping to avoid referencing site 10 (since the range is inclusive, `i=9` would generate `c†(9,s) c(10,s)` which is out of range).

### Yokoyama–Shiba transformation

The **Yokoyama–Shiba (YS) transformation** is a particle–hole transformation applied to **down-spin only**: creation and annihilation for ↓ are swapped (\(c^\dagger_{i\downarrow} \leftrightarrow c_{i\downarrow}\)), while up-spin operators are unchanged. It is often used with Hubbard-type models in mVMC to change the sign structure of the Hamiltonian or to match a particular formulation.

**Rule (down-spin):**

- \(c^\dagger(i,\downarrow) \to c(i,\downarrow)\)
- \(c(i,\downarrow) \to c^\dagger(i,\downarrow)\)

**Usage:**

```bash
quantum-simpl hubbard.def -o output/ --ys-transform
```

With `--ys-transform`, the pipeline also classifies terms into one-body, two-body, and on-site Coulomb (coulomb-intra). When on-site Coulomb terms exist, `coulombintra.def` is written and referenced from `namelist.def`. Any constant (offset) terms are reported on stderr.

### Correlation Function Pipeline

To compute correlation functions (e.g. spin–spin ⟨S·S⟩ or density–density ⟨n n⟩) for mVMC, use a **correlation definition file** with the same DSL. The tool converts spin/density operators to fermion form and writes mVMC measurement inputs.

**Example** — nearest-neighbor spin correlation `S(i)·S(i+1)` on a 4-site chain (`examples/correlation_ss.qsl`):

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
```

Run the correlation pipeline:

```bash
quantum-simpl --correlation examples/correlation_ss.qsl -o output_corr/
```

**Correlation output files** (in the output directory):

| File | Description |
|------|-------------|
| `cisajs.def` | One-body Green’s function form (c†c terms) for mVMC |
| `cisajscktaltdc.def` | Two-body Green’s function form (c†cc†c terms) for mVMC |
| `correlation_summary.txt` | Human-readable list of transformed terms with coefficients |

You can use the same operators as in the Hamiltonian (`S(i).S(j)`, `n(i,s) n(j,s)`, `Sp`, `Sm`, `Sz`, etc.). Coefficients in the correlation file are supported (e.g. `0.5 * S(i) . S(j)`).

## Supported Operators

| Operator | Syntax | Description |
|----------|--------|-------------|
| Creation | `c†(i,spin)` | Fermion creation operator |
| Annihilation | `c(i,spin)` | Fermion annihilation operator |
| Number | `n(i,spin)` | Number operator (sugar for `c†(i,s) c(i,s)`) |
| Spin+ | `Sp(i)` | Spin raising operator |
| Spin- | `Sm(i)` | Spin lowering operator |
| Spin-z | `Sz(i)` | Spin z-component operator |

Spin values: `up`, `down`. Index expressions: `i`, `i+1`, `i-1`, or literal integers. The range `start..end` is **inclusive on both ends** — for N sites with PBC, use `sum i=0..N-1`.

## Output Files

*(Standard pipeline; see "Correlation function pipeline" for `--correlation` outputs.)*

| File | Description |
|------|-------------|
| `namelist.def` | Master index of input files |
| `modpara.def` | Simulation parameters |
| `locspn.def` | Local spin configuration |
| `trans.def` | One-body transfer integrals |
| `interall.def` | Two-body interaction terms |
| `coulombintra.def` | On-site Coulomb terms (written with `--ys-transform` when present) |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |
| `qptransidx.def` | Quantum number projection parameters |

## Documentation

- [Input syntax reference](docs/) (coming soon)

## License

TBD
