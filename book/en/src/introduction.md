# Introduction

**quantum-simpl** is a Hamiltonian symbolic preprocessor that generates [mVMC](https://github.com/issp-center-dev/mVMC) input files from a simple model definition language.

## The Problem

[mVMC](https://github.com/issp-center-dev/mVMC) is a variational Monte Carlo solver for quantum lattice models. It requires several structured input files (`Trans.def`, `InterAll.def`, etc.) to define the Hamiltonian. Writing these by hand is tedious and error-prone, especially for large models with many sites and interactions.

## The Solution

quantum-simpl automates this process. You write a short model definition using a domain-specific language (DSL), and the tool generates all necessary mVMC input files through a five-step pipeline:

1. **Expand** -- unrolls sum loops, expands Hermitian conjugates, desugars `n(i,s)` into `c+(i,s) c(i,s)`
2. **Normal Order** -- applies fermion anticommutation and spin commutation relations to place creation operators before annihilation operators
3. **Combine** -- deduplicates identical operator strings and sums their coefficients
4. **Sz Filter** -- removes terms that violate Sz conservation
5. **Output** -- writes all mVMC input files (`namelist.def`, `trans.def`, `interall.def`, etc.)

The tool supports both **Hubbard** and **Heisenberg** quantum lattice models, as well as a separate **correlation pipeline** for generating measurement input files (e.g., spin-spin or density-density correlation functions).
