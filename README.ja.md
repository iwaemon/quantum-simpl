# quantum-simpl

[English](README.md)

ハミルトニアンシンボリック前処理ツール — モデル定義から [mVMC](https://github.com/issp-center-dev/mVMC) 入力ファイルを自動生成します。

## 機能

- Hubbard・Heisenberg模型を簡潔なDSLで記述
- 演算子項の展開・正規順序化・結合を自動処理
- mVMC入力ファイル一式を生成（`trans.def`、`interall.def` など）
- Yokoyama-Shiba（粒子–空孔）変換に対応
- 相関関数の測定ファイル生成（`cisajs.def`、`cisajscktaltdc.def`）

## クイックスタート

```bash
cargo build --release
```

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

```bash
quantum-simpl hubbard.def -o output/
```

## ドキュメント

詳細なドキュメントは[英語](docs/en/index.html)と[日本語](docs/ja/index.html)で利用できます。

ローカルで閲覧するには：

```bash
cargo install mdbook
mdbook serve book/en    # 英語
mdbook serve book/ja    # 日本語
```

http://localhost:3000 を開いてください。

## ライセンス

未定
