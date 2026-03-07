# Merge Feature Branches & Create Documentation — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Merge feature branches into main, test and fix issues, rewrite READMEs as landing pages, and create bilingual mdBook documentation with built HTML committed to the repo.

**Architecture:** Sequential pipeline — merge first, test to validate, then build documentation on stable code. mdBook uses `[language]` multi-language support with `en/` and `ja/` source directories. Built HTML goes into `docs/` and is committed.

**Tech Stack:** Git, Rust/Cargo (testing), mdBook (documentation)

---

### Task 1: Merge feature/ys-transform into main

**Step 1: Switch to main and merge**

```bash
git checkout main
git merge --no-ff feature/ys-transform -m "Merge feature/ys-transform: YS transform, correlation pipeline, spin-to-fermion"
```

If conflicts occur (likely in README.md, README.ja.md, CLAUDE.md), resolve by accepting the feature branch version — READMEs will be rewritten later.

**Step 2: Verify merge**

```bash
git log --oneline -5
```

Expected: merge commit at HEAD with all feature/ys-transform commits included.

**Step 3: Delete feature branches**

```bash
git branch -d feature/correlation-function
git branch -d feature/ys-transform
```

**Step 4: Commit (already done by merge)**

No additional commit needed.

---

### Task 2: Run cargo test and fix failures

**Step 1: Run all tests**

```bash
cargo test 2>&1
```

Expected: All tests pass. If failures occur, diagnose and fix.

**Step 2: If failures, fix and commit**

Fix each failure, then:

```bash
cargo test
git add <fixed-files>
git commit -m "fix: address test failures after merge"
```

---

### Task 3: Run test-feature skill for E2E validation

**Step 1: Invoke test-feature skill**

Use the `agentic-tests:test-feature` skill to test from a user's perspective.

**Step 2: Fix any issues found**

Address each issue and commit:

```bash
git add <fixed-files>
git commit -m "fix: address issues found in E2E testing"
```

---

### Task 4: Install mdBook

**Step 1: Install mdBook**

```bash
cargo install mdbook
```

**Step 2: Verify installation**

```bash
mdbook --version
```

---

### Task 5: Create mdBook source structure (English)

**Files to create:**
- `book/book.toml`
- `book/src/en/SUMMARY.md`
- `book/src/en/introduction.md`
- `book/src/en/getting-started/installation.md`
- `book/src/en/getting-started/quickstart.md`
- `book/src/en/user-guide/dsl-syntax.md`
- `book/src/en/user-guide/cli-usage.md`
- `book/src/en/user-guide/examples.md`
- `book/src/en/developer-guide/architecture.md`
- `book/src/en/developer-guide/contributing.md`

**Step 1: Create book.toml**

```toml
[book]
title = "quantum-simpl Documentation"
authors = ["quantum-simpl contributors"]

[language.en]
name = "English"
src = "src/en"

[language.ja]
name = "Japanese"
src = "src/ja"

[build]
build-dir = "../docs"
```

**Step 2: Create English SUMMARY.md**

```markdown
# Summary

- [Introduction](introduction.md)

# Getting Started

- [Installation](getting-started/installation.md)
- [Quick Start](getting-started/quickstart.md)

# User Guide

- [DSL Syntax](user-guide/dsl-syntax.md)
- [CLI Usage](user-guide/cli-usage.md)
- [Examples](user-guide/examples.md)

# Developer Guide

- [Architecture](developer-guide/architecture.md)
- [Contributing](developer-guide/contributing.md)
```

**Step 3: Create English content files**

Content sources:
- `introduction.md` — Project overview from current README.md "What is this?" section
- `installation.md` — Install/build instructions from README.md "Install" section
- `quickstart.md` — Basic usage from README.md "Run" section
- `dsl-syntax.md` — Operator table, DSL grammar, sum/params blocks from README.md + CLAUDE.md "DSL Syntax"
- `cli-usage.md` — CLI flags and pipelines from CLAUDE.md "CLI Usage" + README.md "CLI options"
- `examples.md` — Hubbard, Heisenberg, correlation examples from README.md
- `architecture.md` — Pipeline stages, module layout, design choices from CLAUDE.md "Architecture"
- `contributing.md` — Build/test commands, coding conventions from CLAUDE.md "Build & Test Commands"

**Step 4: Commit**

```bash
git add book/
git commit -m "docs: add mdBook English source"
```

---

### Task 6: Create mdBook source structure (Japanese)

**Files to create:**
- `book/src/ja/SUMMARY.md`
- `book/src/ja/introduction.md`
- `book/src/ja/getting-started/installation.md`
- `book/src/ja/getting-started/quickstart.md`
- `book/src/ja/user-guide/dsl-syntax.md`
- `book/src/ja/user-guide/cli-usage.md`
- `book/src/ja/user-guide/examples.md`
- `book/src/ja/developer-guide/architecture.md`
- `book/src/ja/developer-guide/contributing.md`

**Step 1: Create Japanese SUMMARY.md**

Mirror English structure with Japanese titles.

**Step 2: Create Japanese content files**

Translate from English content. Use README.ja.md as a reference for established terminology (演算子、正規順序化、etc.).

**Step 3: Commit**

```bash
git add book/src/ja/
git commit -m "docs: add mdBook Japanese source"
```

---

### Task 7: Build mdBook and commit HTML

**Step 1: Build**

```bash
cd /Users/shumpei/work/test_superpower
mdbook build book/
```

This outputs to `docs/` (configured in book.toml `build-dir = "../docs"`).

**Step 2: Verify build**

```bash
ls docs/
mdbook serve book/ &
# Open http://localhost:3000 to verify (then kill the process)
```

**Step 3: Commit built HTML**

```bash
git add docs/
git commit -m "docs: add built mdBook HTML documentation"
```

---

### Task 8: Rewrite README.md as landing page

**File:** Modify `README.md`

**Step 1: Rewrite content**

```markdown
# quantum-simpl

[日本語](README.ja.md)

A Hamiltonian symbolic preprocessor that generates [mVMC](https://github.com/issp-center-dev/mVMC) input files from a simple model definition.

## Features

- Write Hubbard and Heisenberg models in a concise DSL
- Automatically expands, normal-orders, and combines operator terms
- Generates all required mVMC input files
- Yokoyama-Shiba (particle-hole) transformation support
- Correlation function measurement file generation

## Quick Start

```bash
cargo build --release
quantum-simpl hubbard.def -o output/
```

## Documentation

Full documentation is available in the [docs/](docs/) directory. To browse locally:

```bash
cargo install mdbook
mdbook serve book/
```

Then open http://localhost:3000.

## License

TBD
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: rewrite README.md as concise landing page"
```

---

### Task 9: Rewrite README.ja.md as landing page

**File:** Modify `README.ja.md`

**Step 1: Rewrite content**

Japanese translation of the landing page README, mirroring Task 8 structure.

**Step 2: Commit**

```bash
git add README.ja.md
git commit -m "docs: rewrite README.ja.md as concise landing page"
```

---

### Task 10: Final verification

**Step 1: Run all tests**

```bash
cargo test
```

**Step 2: Verify mdBook builds cleanly**

```bash
mdbook build book/
```

**Step 3: Check git status is clean**

```bash
git status
git log --oneline -15
```
