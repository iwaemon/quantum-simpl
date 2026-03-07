# CLI使用方法

## 基本的な使い方

```bash
quantum-simpl <INPUT> -o <OUTPUT_DIR>
```

- `<INPUT>` -- DSL入力ファイルのパス
- `-o <OUTPUT_DIR>` -- 出力ディレクトリ（存在しない場合は自動作成）

## 標準パイプライン

ハミルトニアン定義から mVMC 入力ファイルを生成します：

```bash
quantum-simpl hubbard.def -o output/
```

## Yokoyama--Shiba変換

`--ys-transform` フラグを付けると、ダウンスピンに対して粒子--空孔変換を適用します：

```bash
quantum-simpl hubbard.def -o output/ --ys-transform
```

この変換では以下が行われます：
- ダウンスピンの生成演算子と消滅演算子を入れ替え（\\(c^\dagger_{i\downarrow} \leftrightarrow c_{i\downarrow}\\)）
- アップスピンはそのまま
- 項を一体・二体・オンサイトCoulomb（coulomb-intra）に分類
- オンサイトCoulomb項がある場合は `coulombintra.def` を出力
- 定数項（エネルギーオフセット）は標準エラーに表示

## 相関関数パイプライン

`--correlation` フラグで相関関数の測定用入力ファイルを生成します：

```bash
quantum-simpl --correlation corr.def -o output/
```

出力ファイル：
- `cisajs.def` -- 2体Green関数の測定定義
- `cisajscktaltdc.def` -- 4体Green関数の測定定義
- `correlation_summary.txt` -- 人間可読な変換結果

## 両パイプラインの同時実行

ハミルトニアンと相関関数を同時に処理することもできます：

```bash
quantum-simpl input.def --correlation corr.def -o output/
```

## ヘルプ

```bash
quantum-simpl --help
```
