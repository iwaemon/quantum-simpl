# README Design

## Decisions

- **Audience**: Physicists/researchers who want to use quantum-simpl
- **mVMC context**: Brief explanation of what mVMC is, then focus on tool usage
- **Input syntax docs**: Quick example in README, detailed reference in separate docs
- **Language**: English (README.md) + Japanese (README.ja.md), linked to each other
- **Install method**: `cargo install` / `cargo build` only
- **Style**: User-centric — focus on getting started quickly

## README.md (English)

```
# quantum-simpl

One-line description + language switcher link to README.ja.md

## What is this?
- 2-3 sentences on mVMC (variational Monte Carlo solver for quantum lattice models)
- quantum-simpl generates mVMC input files from a simple model definition DSL
- Pipeline: Expand → Normal Order → Combine → Sz Filter → Output

## Quick Start
- cargo install / cargo build --release
- Minimal Hubbard model DSL example (~5 lines)
- Run command: quantum-simpl input.def -o output/
- Brief description of what gets generated

## Output Files
- List of generated files (Trans.def, InterAll.def, modpara.def, etc.)
- One-line description of each

## Documentation
- Link to detailed input syntax reference (docs/)

## License
- Placeholder (not yet decided)
```

## README.ja.md (Japanese)

Same structure as README.md, translated to Japanese:

- Language switcher link to README.md at top
- これは何？ / クイックスタート / 出力ファイル / ドキュメント / ライセンス
- Same Hubbard example code (not translated — code stays in English)
