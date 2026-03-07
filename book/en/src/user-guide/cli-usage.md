# CLI Usage

## Standard Pipeline

Generate mVMC Hamiltonian input files from a model definition:

```bash
quantum-simpl <INPUT> -o <DIR>
```

- `<INPUT>` -- path to the model definition file
- `-o <DIR>` -- output directory for generated files

### Pipeline stages

```
Parse -> Expand -> Spin->Fermion -> Normal Order -> Combine -> Sz Filter -> mVMC Output
```

## Yokoyama-Shiba Transform

Apply the Yokoyama-Shiba (particle-hole) transformation for down-spin:

```bash
quantum-simpl <INPUT> -o <DIR> --ys-transform
```

The YS transformation swaps creation and annihilation operators for down-spin only:

- `c+(i,down)` becomes `c(i,down)`
- `c(i,down)` becomes `c+(i,down)`

Up-spin operators are unchanged.

With `--ys-transform`, the pipeline additionally:

1. Classifies terms into one-body, coulomb-intra, and two-body categories
2. Writes `coulombintra.def` when on-site Coulomb terms are present
3. Reports any constant (offset) terms on stderr

### Pipeline stages (YS)

```
Parse -> Expand -> Spin->Fermion -> YS Transform -> Normal Order -> Combine -> Sz Filter -> Classify -> mVMC Output
```

## Correlation Pipeline

Generate mVMC measurement input files for correlation functions:

```bash
quantum-simpl --correlation <FILE> -o <DIR>
```

The correlation file uses the same DSL syntax. Spin and density operators are converted to fermion form and written as Green's function measurement inputs.

### Pipeline stages (correlation)

```
Parse -> Expand -> Spin->Fermion -> Normal Order -> Combine -> Green Reorder -> cisajs/cisajscktaltdc Output
```

## Combined Mode

Run both the standard and correlation pipelines together:

```bash
quantum-simpl <INPUT> --correlation <FILE> -o <DIR>
```

Both sets of output files are written to the same directory.

## Output Files

### Standard pipeline output

| File | Description |
|------|-------------|
| `namelist.def` | Master index of input files |
| `modpara.def` | Simulation parameters |
| `locspn.def` | Local spin configuration |
| `trans.def` | One-body transfer integrals |
| `interall.def` | Two-body interaction terms |
| `coulombintra.def` | On-site Coulomb terms (only with `--ys-transform`) |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |
| `qptransidx.def` | Quantum number projection parameters |

### Correlation pipeline output

| File | Description |
|------|-------------|
| `cisajs.def` | One-body Green's function terms (c+c) |
| `cisajscktaltdc.def` | Two-body Green's function terms (c+c c+c) |
| `correlation_summary.txt` | Human-readable list of transformed terms |
