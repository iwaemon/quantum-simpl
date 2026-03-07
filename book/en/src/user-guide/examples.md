# Examples

## Hubbard Model

A 10-site Hubbard chain with periodic boundary conditions, hopping parameter `t=1.0` and on-site interaction `U=4.0`.

**Input file** (`hubbard.def`):

```
lattice 1d sites=10 pbc=true

sum i=0..9:
  -t * c+(i,up) c(i+1,up) + h.c.
  -t * c+(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
```

**Run:**

```bash
quantum-simpl hubbard.def -o output/
```

**Output:** The `output/` directory contains `namelist.def`, `modpara.def`, `trans.def` (20 hopping terms for each spin channel), `interall.def` (10 on-site U terms), and variational parameter files.

## Heisenberg Model

A 10-site antiferromagnetic Heisenberg chain using the `S(i).S(j)` shorthand for spin-spin interaction.

**Input file** (`heisenberg.qsl`):

```
lattice 1d sites=10 pbc=true

sum i=0..9:
  J * S(i) . S(i+1)

params:
  J = 1.0
```

The `S(i).S(j)` notation is syntactic sugar that expands to:

```
0.5 * Sp(i) Sm(j) + 0.5 * Sm(i) Sp(j) + 1.0 * Sz(i) Sz(j)
```

All spin operators are then converted to fermion operators internally.

**Run:**

```bash
quantum-simpl heisenberg.qsl -o output/
```

**Output:** The same set of mVMC input files. The Heisenberg interaction produces both `trans.def` and `interall.def` entries after spin-to-fermion conversion and normal ordering.

## Spin-Spin Correlation Measurement

Measure nearest-neighbor spin correlations on a 4-site chain.

**Correlation file** (`correlation_ss.qsl`):

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
```

**Run:**

```bash
quantum-simpl --correlation correlation_ss.qsl -o output_corr/
```

**Output:** The `output_corr/` directory contains:

- `cisajs.def` -- one-body Green's function terms arising from anticommutation corrections during Green reordering
- `cisajscktaltdc.def` -- two-body Green's function terms in the `c+c c+c` form required by mVMC
- `correlation_summary.txt` -- human-readable summary showing how each spin operator product was transformed into fermion operators with coefficients

You can also combine the correlation pipeline with the standard pipeline:

```bash
quantum-simpl hubbard.def --correlation correlation_ss.qsl -o output/
```

This generates both the Hamiltonian input files and the measurement files in the same output directory.
