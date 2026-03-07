# Quick Start

## 1. Create an input file

Create a file called `hubbard.def` with the following content -- a 10-site Hubbard model with periodic boundary conditions:

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

## 2. Run quantum-simpl

```bash
quantum-simpl hubbard.def -o output/
```

## 3. Check the output

The `output/` directory now contains the mVMC input files:

| File | Description |
|------|-------------|
| `namelist.def` | Master index referencing all other input files |
| `modpara.def` | Simulation parameters (number of sites, electrons, etc.) |
| `locspn.def` | Local spin configuration for each site |
| `trans.def` | One-body transfer integrals (hopping terms) |
| `interall.def` | Two-body interaction terms (e.g., Hubbard U) |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |
| `qptransidx.def` | Quantum number projection parameters |

These files can be used directly as input to mVMC.
