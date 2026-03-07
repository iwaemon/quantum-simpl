# DSL Syntax

quantum-simpl uses a line-based domain-specific language for defining Hamiltonians and correlation functions. Input files can use any extension (`.qsl`, `.def`, etc.).

## Lattice Declaration

Every input file begins with a lattice declaration:

```
lattice 1d sites=N pbc=true|false
```

- `sites=N` -- number of lattice sites (integer)
- `pbc=true` -- periodic boundary conditions (sites wrap around)
- `pbc=false` -- open boundary conditions (out-of-range terms are dropped with a warning)

## Sum Blocks

Sum blocks define terms that are repeated over a range of site indices:

```
sum var=start..end:
  expression
  expression
  ...
```

- `var` -- loop variable name (typically `i`, `j`, etc.)
- `start..end` -- **inclusive on both ends**. For N sites with PBC, use `0..N-1`.
- Each indented line after the colon is a separate term in the Hamiltonian.

Multiple sum blocks can appear in a single file.

## Operators

| Operator | Syntax | Description |
|----------|--------|-------------|
| Creation | `c+(i,spin)` | Fermion creation operator |
| Annihilation | `c(i,spin)` | Fermion annihilation operator |
| Number | `n(i,spin)` | Number operator (syntactic sugar for `c+(i,s) c(i,s)`) |
| Spin raising | `Sp(i)` | Spin-+ operator |
| Spin lowering | `Sm(i)` | Spin-- operator |
| Spin z-component | `Sz(i)` | Spin-z operator |
| Heisenberg dot | `S(i) . S(j)` | Expands to `0.5*Sp(i)Sm(j) + 0.5*Sm(i)Sp(j) + Sz(i)Sz(j)` |

### Spin values

Spin arguments are `up` or `down`. Spin operators (`Sp`, `Sm`, `Sz`, `S.S`) do not take a spin argument -- they operate on both spin channels internally.

### Index expressions

Operator indices support the following forms:

- `var` -- the loop variable itself (e.g., `i`)
- `var+offset` -- variable plus an integer offset (e.g., `i+1`)
- `var-offset` -- variable minus an integer offset (e.g., `i-1`)
- Literal integers (e.g., `0`, `5`)

With periodic boundary conditions, index arithmetic wraps modulo the number of sites.

## Coefficients and Expressions

Each term line has the form:

```
coeff * operator operator ... [+ h.c.]
```

- `coeff` -- a numeric literal (e.g., `-1.0`, `0.5`) or a parameter name (e.g., `t`, `U`, `J`)
- Operators are written in sequence (implicit multiplication)
- `+ h.c.` at the end of a line adds the Hermitian conjugate of the term

## Hermitian Conjugate

Appending `+ h.c.` to a term line adds the conjugate transpose: creation and annihilation operators are swapped and the coefficient is complex-conjugated. This is commonly used for hopping terms:

```
-t * c+(i,up) c(i+1,up) + h.c.
```

This expands to both `-t * c+(i,up) c(i+1,up)` and `-t * c+(i+1,up) c(i,up)`.

## Parameters Section

Named parameters are defined at the end of the file:

```
params:
  name = value
```

Parameter names used in term coefficients are substituted with the corresponding numeric values during expansion.

## Open Boundary Conditions

When using `pbc=false`, terms that reference sites outside the valid range `[0, N-1]` are silently dropped. Adjust sum ranges accordingly -- for example, with `sites=10 pbc=false`, use `sum i=0..8` for nearest-neighbor terms to avoid referencing site 10.
