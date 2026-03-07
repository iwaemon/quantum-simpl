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
| `namelist.def` | 入力ファイルのマスターインデックス |
| `modpara.def` | シミュレーションパラメータ |
| `locspn.def` | 局所スピン設定 |
| `trans.def` | 一体遷移積分 |
| `interall.def` | 二体相互作用項 |
| `gutzwilleridx.def` | Gutzwiller変分パラメータ |
| `jastrowidx.def` | Jastrow変分パラメータ |
| `orbitalidx.def` | 軌道変分パラメータ |
| `qptransidx.def` | 量子数射影パラメータ |

## ドキュメント

- [入力構文リファレンス](docs/)（準備中）

## ライセンス

未定
