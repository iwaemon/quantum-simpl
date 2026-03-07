# 使用例

## 1次元Hubbard模型（周期的境界条件）

10サイトの1次元Hubbard模型。最近接ホッピング \\(t=1.0\\)、オンサイト相互作用 \\(U=4.0\\)。

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

## 1次元Heisenberg模型

10サイトの反強磁性Heisenberg模型。

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

## 開放境界条件のHubbard模型

`pbc=false` では、sumの範囲に注意が必要です。最近接ホッピングの場合、最後のサイトから次のサイトへの項は存在しないため、範囲を1つ短くします。

```
lattice 1d sites=10 pbc=false

sum i=0..8:
  -t * c†(i,up) c(i+1,up) + h.c.
  -t * c†(i,down) c(i+1,down) + h.c.

sum i=0..9:
  U * n(i,up) n(i,down)

params:
  t = 1.0
  U = 4.0
```

## Yokoyama--Shiba変換付きHubbard模型

mVMCの特定の定式化に合わせるため、ダウンスピンに粒子--空孔変換を適用します。

```bash
quantum-simpl hubbard.def -o output/ --ys-transform
```

`--ys-transform` を付けると、通常の出力に加えて `coulombintra.def` が生成される場合があります。定数項（エネルギーオフセット）は標準エラーに表示されます。

## スピン相関関数の測定

スピン相関 \\(\langle \mathbf{S}_i \cdot \mathbf{S}_j \rangle\\) の測定用ファイルを生成します。

相関関数定義ファイル `corr.qsl` を作成します：

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  S(i) . S(i+1)
```

```bash
quantum-simpl --correlation corr.qsl -o output/
```

内部では `S(i) . S(j)` がフェルミオン形式 `c†c c†c` に自動変換され、`cisajs.def` と `cisajscktaltdc.def` が生成されます。

## 密度相関関数の測定

密度相関 \\(\langle n_{i\uparrow} n_{j\uparrow} \rangle\\) の測定も同様に記述できます：

```
lattice 1d sites=4 pbc=true

sum i=0..3:
  n(i,up) n(i+1,up)
```

```bash
quantum-simpl --correlation density_corr.qsl -o output/
```
