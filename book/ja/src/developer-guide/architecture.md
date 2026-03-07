# アーキテクチャ

## 2つのパイプライン

CLI（`src/main.rs`）にはフラグで選択される2つのパイプラインがあります。

### 標準パイプライン

デフォルトのパイプラインで、`<INPUT>` を処理します：

```
Parse → Expand → Spin→Fermion → [YS Transform] → Normal Order
  → Combine → Sz Filter → [Classify] → mVMC Output
```

YS変換パスでは、項の分類（一体/coulomb-intra/二体）も行い、該当する場合は `coulombintra.def` を出力します。

### 相関関数パイプライン

`--correlation <FILE>` で起動します：

```
Parse → Expand → Spin→Fermion → Normal Order → Combine
  → Green Reorder → cisajs/cisajscktaltdc Output
```

GreenReorder段階では、4演算子の項を Green関数形式（`c†c c†c`）に並べ替え、反交換関係に基づくデルタ補正項を生成します。

## モジュール構成

### `src/parser/` -- パーサー

手書きの行ベースDSLパーサーです。

- `ast.rs` -- 構文木の型定義（`ModelDef`、`SumBlock`、`Expression`、`OpExpr` など）
- `mod.rs` -- パーサー本体のロジック

### `src/core/` -- 変換パイプライン

各段階が独立したモジュールとして実装されています：

| モジュール | 役割 |
|-----------|------|
| `op.rs` | 基本データ型: `Op`（フェルミオン/スピン演算子のenum）、`Term`（係数 + 演算子列）、`Hamiltonian` |
| `expand.rs` | sumループ展開、エルミート共役展開、`n(i,s)` の脱糖、パラメータ代入 |
| `normal.rs` | フェルミオンの反交換関係を適用して正規順序化（生成演算子を左に） |
| `combine.rs` | ハッシュベースの同一演算子列の重複除去、係数の和 |
| `symmetry.rs` | Sz保存を破る項のフィルタリング |
| `transform.rs` | 置換規則: 粒子--空孔変換（YS変換）、スピン→フェルミオン変換（`Sp/Sm/Sz` → `c†c`） |
| `classify.rs` | 項を定数・一体・coulomb-intra・二体に分類（YSパスで使用） |
| `green.rs` | 4演算子項をGreen関数形式（`c†c c†c`）に並べ替え、反交換関係の補正 |

### `src/output/` -- 出力

- `mvmc.rs` -- mVMC形式の `.def` ファイル書き出し（namelist、modpara、trans、interall、cisajs、cisajscktaltdc など）
- `correlation.rs` -- `correlation_summary.txt` の人間可読フォーマッタ

## 設計上の選択

### フラットな項テーブル

項は式木ではなく `Vec<Term>` として格納されます。10万項以上を扱う場合にキャッシュ局所性が向上します。

### `SmallVec<[Op; 4]>`

Hubbard/Heisenberg模型のほとんどの項は演算子4個以下であるため、`SmallVec` によりスタック上に配置され、ヒープ割り当てを回避します。

### `FxHashMap`

`rustc-hash` クレートの非暗号学的ハッシュマップを結合段階で使用し、高速なハッシュ処理を実現しています。

## テスト構成

- `tests/unit_*.rs` -- 各パイプライン段階のユニットテスト（parser、expand、normal、combine、symmetry、op、mvmc、transform、classify、green、correlation）
- `tests/integration/` -- エンドツーエンドテスト（`test_pipeline`、`test_hubbard`、`test_heisenberg`、`test_mvmc_output`、`test_ys_transform`、`test_ys_validation`、`test_correlation`）
