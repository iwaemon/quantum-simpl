# クイックスタート

## Hubbard模型

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

`output/` ディレクトリに mVMC 入力ファイル一式が生成されます。

## Heisenberg模型

スピン演算子を使った Heisenberg 模型も記述できます。`heisenberg.qsl` を作成します：

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

内部では `S(i) . S(j)` が `0.5*Sp(i)Sm(j) + 0.5*Sm(i)Sp(j) + Sz(i)Sz(j)` に展開され、さらにフェルミオン演算子に変換されて出力されます。

## 生成されるファイル

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

入力ファイルの拡張子は任意です（`.qsl`、`.def` など）。
