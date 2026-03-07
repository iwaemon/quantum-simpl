# スピン相関関数→フェルミオン自動変換 設計

**日付:** 2026-03-07
**ステータス:** 承認済み

## 目的

`⟨S_i·S_j⟩` 等のスピン相関関数をフェルミオン形式 `⟨c†c c†c⟩` に自動変換し、mVMCの測定用入力ファイル（`cisajs.def`, `cisajscktaltdc.def`）を生成する。

## 入力

既存DSL文法を再利用した独立ファイル（`correlation.qsl`）:

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
  n(i,up) n(i+1,up)
```

CLI: `quantum-simpl --correlation correlation.qsl -o output`

## 文法追加

`S(i) . S(j)` シンタックスシュガー。パーサーが以下の3式に展開:

```
0.5 * Sp(i) Sm(j)
0.5 * Sm(i) Sp(j)
1.0 * Sz(i) Sz(j)
```

`n(i,s) n(j,s)` や `c†(i,s) c(j,s)` 等は既存文法のまま使用可能。

## パイプライン

```
Parse → Expand (sumループ展開)
  → SpinToFermion 変換
      Sp(i) → c†(i,↑) c(i,↓)
      Sm(i) → c†(i,↓) c(i,↑)
      Sz(i) → 0.5*c†(i,↑)c(i,↑) - 0.5*c†(i,↓)c(i,↓)  ※Term分裂あり
  → NormalOrder (c†を左に)
  → Combine (同一項をまとめる)
  → GreenReorder (c†c†cc → c†cc†c + δ補正)
  → 出力
```

### SpinToFermion 変換

`transform.rs` に `spin_to_fermion(terms: &[Term]) -> Vec<Term>` を追加。

- `Sp`/`Sm` は演算子の1対1置換（Op→Op）
- `Sz` は1つのTermが複数Termに分裂する（`0.5*n↑ - 0.5*n↓`）ため、既存の `apply_substitution`（op単位）とは別の専用関数として実装

### GreenReorder

既存計画 Task 6 (`green.rs`) をそのまま活用。`c†c†cc → c†cc†c + δ·c†c` の変換を行う。

## 出力

| ファイル | 内容 |
|---------|------|
| `cisajscktaltdc.def` | 4体 Green 関数（c†cc†c 形式） |
| `cisajs.def` | 2体 Green 関数（δ補正由来） |
| `correlation_summary.txt` | 人間可読な変換結果（symbolic表示） |

### correlation_summary.txt の例

```
# S(0) . S(1) → 6 fermionic terms
  +0.50 * c†(0,↑) c(0,↓) c†(1,↓) c(1,↑)    # from Sp(0)Sm(1)
  +0.50 * c†(0,↓) c(0,↑) c†(1,↑) c(1,↓)    # from Sm(0)Sp(1)
  +0.25 * c†(0,↑) c(0,↑) c†(1,↑) c(1,↑)    # from Sz(0)Sz(1)
  -0.25 * c†(0,↑) c(0,↑) c†(1,↓) c(1,↓)    # from Sz(0)Sz(1)
  -0.25 * c†(0,↓) c(0,↓) c†(1,↑) c(1,↑)    # from Sz(0)Sz(1)
  +0.25 * c†(0,↓) c(0,↓) c†(1,↓) c(1,↓)    # from Sz(0)Sz(1)
```

## 初期サポート

- `S(i).S(j)` — スピン相関
- `n(i,s) n(j,s)` — 密度相関
- 任意の `c†/c` 演算子積 — 既存文法でそのまま記述可能

将来的には汎用的な演算子積に拡張可能（DSLで書ける任意の式を相関関数として出力）。

## 既存計画との関係

- Task 6 (`green.rs`) の GreenReorder を活用
- `transform.rs` に `SpinToFermion` を追加
- パーサーに `S(i).S(j)` シュガーを追加

## アーキテクチャ選択理由

既存パイプライン（Parser → Expand → NormalOrder → Combine）を最大限再利用するアプローチを採用。sumループ、PBC、パラメータ置換が全てそのまま使える。新規コードは SpinToFermion 変換、GreenReorder、出力フォーマッタの3点のみ。
