# quantum-simpl

[日本語](README.ja.md)

A Hamiltonian symbolic preprocessor that generates [mVMC](https://github.com/issp-center-dev/mVMC) input files from a simple model definition.

## Features

- Write Hubbard and Heisenberg models in a concise DSL
- Automatically expands, normal-orders, and combines operator terms
- Generates all required mVMC input files (`trans.def`, `interall.def`, etc.)
- Yokoyama-Shiba (particle-hole) transformation support
- Correlation function measurement file generation (`cisajs.def`, `cisajscktaltdc.def`)

## Quick Start

```bash
cargo build --release
```

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

```bash
quantum-simpl hubbard.def -o output/
```

## Documentation

Full documentation is available in [English](docs/en/index.html) and [Japanese](docs/ja/index.html).

To browse locally:

```bash
cargo install mdbook
mdbook serve book/en    # English
mdbook serve book/ja    # Japanese
```

Then open http://localhost:3000.

## License

TBD
