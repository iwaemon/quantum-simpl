# コントリビューション

quantum-simpl への貢献を歓迎します。

## 開発環境のセットアップ

```bash
git clone <repository-url>
cd quantum-simpl
cargo build
cargo test
```

全テストが通ることを確認してから開発を始めてください。

## ビルドとテスト

```bash
cargo build                          # デバッグビルド
cargo build --release                # リリースビルド
cargo test                           # 全テスト実行
cargo test test_pipeline             # 特定の統合テスト
cargo test unit_parser               # 特定のユニットテスト
cargo test --test test_hubbard       # テストファイル名で指定
```

## コードの構成

新しい機能を追加する際の指針：

### パイプライン段階の追加

1. `src/core/` に新しいモジュールを作成
2. `fn transform(terms: &[Term]) -> Vec<Term>` のようなシグネチャで変換関数を実装
3. `src/main.rs` のパイプラインに組み込み
4. `tests/unit_<module>.rs` にユニットテストを追加

### 新しい演算子の追加

1. `src/core/op.rs` の `Op` enumにバリアントを追加
2. `src/parser/` にパース処理を追加
3. 必要に応じて `transform.rs` に変換ルールを追加
4. `normal.rs` に正規順序化のルールを追加

### 出力フォーマットの追加

1. `src/output/` に新しいモジュールを作成
2. `src/main.rs` から呼び出し

## テストの書き方

- ユニットテストは `tests/unit_<module>.rs` に配置
- 統合テストは `tests/integration/` に配置
- パイプライン全体を通すテストは `test_pipeline` パターンを参考にしてください

## コーディング規約

- `cargo fmt` でフォーマット
- `cargo clippy` で警告がないことを確認
- パブリックな関数にはドキュメントコメントを付ける
