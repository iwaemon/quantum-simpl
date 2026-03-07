# quantum-simpl

[English](README.md)

ハミルトニアンシンボリック前処理ツール — モデル定義から [mVMC](https://github.com/issp-center-dev/mVMC) 入力ファイルを自動生成します。

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

sum i=0..9:
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

`output/` ディレクトリに mVMC 入力ファイルが生成されます。入力ファイルの拡張子は任意です（`.qsl`、`.def` など）。

### Heisenbergモデルの例

quantum-simpl はスピン演算子にも対応しています。`heisenberg.qsl` を作成します：

```
lattice 1d sites=10 pbc=true

sum i=0..9:
  J * S(i) . S(i+1)

params:
  J = 1.0
```

```bash
quantum-simpl heisenberg.qsl -o output/
```

### 開放境界条件に関する注意

`pbc=false` を使用する場合、格子範囲外のサイトを参照する項は警告付きで除去されます。sumの範囲が格子サイズと整合するよう注意してください。例えば `sites=10 pbc=false` の場合、最近接ホッピングには `sum i=0..8` を使用してください（範囲は両端を含むため、`i=9` は `c†(9,s) c(10,s)` を生成し、範囲外となります）。

### Yokoyama–Shiba 変換

**Yokoyama–Shiba（YS）変換**は、**ダウンスピンだけ**に粒子–空孔変換をかけるものです：↓の生成・消滅を入れ替え（\(c^\dagger_{i\downarrow} \leftrightarrow c_{i\downarrow}\)）、アップスピンはそのままにします。Hubbard型モデルでmVMCの特定の定式化に合わせるときなどに使います。

**変換則（ダウンスピンのみ）:**

- \(c^\dagger(i,\downarrow) \to c(i,\downarrow)\)
- \(c(i,\downarrow) \to c^\dagger(i,\downarrow)\)

**使い方:**

```bash
quantum-simpl hubbard.def -o output/ --ys-transform
```

`--ys-transform` を付けると、項を一体・二体・オンサイトCoulomb（coulomb-intra）に分類し、オンサイトCoulombがある場合は `coulombintra.def` を出力して `namelist.def` から参照します。定数項（オフセット）は標準エラーに表示されます。

## 対応する演算子

| 演算子 | 構文 | 説明 |
|--------|------|------|
| 生成 | `c†(i,spin)` | フェルミオン生成演算子 |
| 消滅 | `c(i,spin)` | フェルミオン消滅演算子 |
| 数 | `n(i,spin)` | 数演算子（`c†(i,s) c(i,s)` の糖衣構文） |
| スピン+ | `Sp(i)` | スピン上昇演算子 |
| スピン- | `Sm(i)` | スピン下降演算子 |
| スピンz | `Sz(i)` | スピンz成分演算子 |

スピン値: `up`、`down`。インデックス式: `i`、`i+1`、`i-1`、またはリテラル整数。範囲 `start..end` は**両端を含みます** — N サイトの PBC では `sum i=0..N-1` を使用してください。

## 出力ファイル

| ファイル | 説明 |
|----------|------|
| `namelist.def` | 入力ファイルのマスターインデックス |
| `modpara.def` | シミュレーションパラメータ |
| `locspn.def` | 局所スピン設定 |
| `trans.def` | 一体遷移積分 |
| `interall.def` | 二体相互作用項 |
| `coulombintra.def` | オンサイトCoulomb項（`--ys-transform` 時、該当項がある場合に出力） |
| `gutzwilleridx.def` | Gutzwiller変分パラメータ |
| `jastrowidx.def` | Jastrow変分パラメータ |
| `orbitalidx.def` | 軌道変分パラメータ |
| `qptransidx.def` | 量子数射影パラメータ |

## ドキュメント

- [入力構文リファレンス](docs/)（準備中）

## ライセンス

未定
