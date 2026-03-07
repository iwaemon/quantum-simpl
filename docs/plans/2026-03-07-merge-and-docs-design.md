# Design: Merge Feature Branches, Test, and Create Documentation

Date: 2026-03-07

## Overview

Merge scattered feature branches into main, test and fix issues, rewrite READMEs as concise landing pages, and create comprehensive mdBook documentation (EN/JA) with built HTML committed to the repository.

## 1. Merge Strategy

- Merge `feature/ys-transform` into main with `--no-ff` (merge commit)
- `feature/correlation-function` is a subset of `feature/ys-transform`, so no separate merge needed
- Delete both feature branches after merge
- Resolve conflicts by preferring ys-transform side (READMEs will be rewritten anyway)

## 2. Test and Fix

- Run `cargo test` after merge to verify all tests pass
- Use test-feature skill for user-perspective E2E testing
- Fix discovered issues and commit

## 3. README.md / README.ja.md

Rewrite both as concise landing pages:

- Project name + one-line description
- Key features (3-5 bullet points)
- Quick start (install + basic commands)
- Link to mdBook documentation in `docs/`
- License

All detailed content (DSL syntax, architecture, etc.) moves to mdBook.

## 4. mdBook Structure

### Directory Layout

```
book/                    # mdBook source
  book.toml
  src/
    en/                  # English content
      SUMMARY.md
      introduction.md
      getting-started/
        installation.md
        quickstart.md
      user-guide/
        dsl-syntax.md
        cli-usage.md
        examples.md
      developer-guide/
        architecture.md
        contributing.md
    ja/                  # Japanese content
      SUMMARY.md
      introduction.md
      getting-started/
        installation.md
        quickstart.md
      user-guide/
        dsl-syntax.md
        cli-usage.md
        examples.md
      developer-guide/
        architecture.md
        contributing.md
docs/                    # mdbook build output (committed HTML)
```

### Multilingual Support

Use mdBook's `[language]` table in `book.toml` with `en/` and `ja/` subdirectories.

### Content Sources

- Existing CLAUDE.md (architecture, module layout, CLI usage)
- Current README.md / README.ja.md (DSL syntax, examples)
- docs/plans/ design documents (correlation function, YS transform details)

## Execution Order

1. Merge `feature/ys-transform` → main
2. Delete feature branches
3. Test (cargo test + test-feature skill) → fix
4. Rewrite README.md / README.ja.md
5. Create mdBook source in `book/`
6. `mdbook build` → output to `docs/`
7. Commit built HTML
