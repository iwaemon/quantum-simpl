# README Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create README.md (English) and README.ja.md (Japanese) for quantum-simpl.

**Architecture:** Two static markdown files with cross-links. Content follows design in `docs/plans/2026-03-07-readme-design.md`.

**Tech Stack:** Markdown only.

---

### Task 1: Create README.md (English)

**Files:**
- Create: `README.md`

**Step 1: Write README.md**

```markdown
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

sum i=0..10:
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

mVMC input files are generated in the `output/` directory.

## Output Files

| File | Description |
|------|-------------|
| `Trans.def` | One-body transfer integrals |
| `InterAll.def` | Two-body interaction terms |
| `modpara.def` | Simulation parameters |
| `locspn.def` | Local spin configuration |
| `gutzwilleridx.def` | Gutzwiller variational parameters |
| `jastrowidx.def` | Jastrow variational parameters |
| `orbitalidx.def` | Orbital variational parameters |

## Documentation

- [Input syntax reference](docs/) (coming soon)

## License

TBD
```

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add README.md (English)"
```

---

### Task 2: Create README.ja.md (Japanese)

**Files:**
- Create: `README.ja.md`

**Step 1: Write README.ja.md**

```markdown
# quantum-simpl

[English](README.md)

ハミルトニアン記号前処理ツール — モデル定義から [mVMC](https://github.com/issp-center-dev/mVMC) 入力ファイルを自動生成します。

## これは何？

[mVMC](https://github.com/issp-center-dev/mVMC) は量子格子模型の変分モンテカルロソルバーです。ハミルトニアンを定義するために複数の構造化された入力ファイル（`Trans.def`、`InterAll.def` など）が必要ですが、大規模なモデルではこれらを手作業で書くのは煩雑でミスが起きやすい作業です。

quantum-simpl はこれを自動化します。短いモデル定義を書くだけで、以下の処理を行います：

1. **展開** — sumループの展開、エルミート共役の展開、`n(i,s)` の脱糖
2. **正規順序化** — フェルミオンの反交換関係とスピンの交換関係を適用
3. **結合** — 同一演算子列の重複除去、係数の和
4. **フィルタ** — Sz保存を破る項の除去
5. **出力** — mVMC入力ファイルの書き出し

## クイックスタート

### インストール

```bash
cargo install --path .
```

ソースからビルドする場合：

```bash
cargo build --release
# バイナリは target/release/quantum-simpl に生成されます
```

### 実行

入力ファイル `hubbard.def` を作成します：

```
lattice 1d sites=10 pbc=true

sum i=0..10:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
```

quantum-simpl を実行します：

```bash
quantum-simpl hubbard.def -o output/
```

`output/` ディレクトリに mVMC 入力ファイルが生成されます。

## 出力ファイル

| ファイル | 説明 |
|----------|------|
| `Trans.def` | 一体移行積分 |
| `InterAll.def` | 二体相互作用項 |
| `modpara.def` | シミュレーションパラメータ |
| `locspn.def` | 局所スピン設定 |
| `gutzwilleridx.def` | Gutzwiller変分パラメータ |
| `jastrowidx.def` | Jastrow変分パラメータ |
| `orbitalidx.def` | 軌道変分パラメータ |

## ドキュメント

- [入力構文リファレンス](docs/)（準備中）

## ライセンス

未定
```

**Step 2: Commit**

```bash
git add README.ja.md
git commit -m "docs: add README.ja.md (Japanese)"
```
