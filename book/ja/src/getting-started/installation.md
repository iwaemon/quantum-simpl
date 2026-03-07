# インストール

## 必要環境

- **Rust** 1.70 以上（[rustup](https://rustup.rs/) でインストール推奨）

## cargo install によるインストール

```bash
cargo install --path .
```

これにより `quantum-simpl` バイナリが `~/.cargo/bin/` にインストールされます。

## ソースからビルド

```bash
git clone <repository-url>
cd quantum-simpl
cargo build --release
```

リリースバイナリは `target/release/quantum-simpl` に生成されます。

## 動作確認

```bash
quantum-simpl --help
```

ヘルプメッセージが表示されれば、インストールは完了です。

## テストの実行

全テストを実行して、ビルドが正常であることを確認できます：

```bash
cargo test
```
