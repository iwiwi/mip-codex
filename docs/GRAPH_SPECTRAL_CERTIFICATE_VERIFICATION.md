# 元MPSでの整数スペクトル証明の検査

`result.json`は`mip-codex solve model.mps.gz 10`の標準出力とする。点とすべての境界証明を別に検査する。

```sh
python3 scripts/verify_cover_cuts.py model.mps.gz result.json
MIP_ABSOLUTE_CHECKER="$PWD/third_party/miplib-checker/bin/solchecker-absolute" \
  .venv/bin/python scripts/benchmark_validate.py model.mps.gz result.json \
  third_party/miplib-checker/bin/solchecker validation-output
```

第1コマンドは元MPSのFractionを使い、packing認識、根彩色・被覆・残prefix・スペクトル証明を検査する。第2コマンドは元制約、変数境界、整数性、目的をGMP絶対検査（制約/整数1e−6、目的相対1e−8）と公式補助検査で照合する。

スペクトル証明だけを取り出す場合:

```sh
jq '.binary_graph.spectral_bound.certificate' result.json > spectral-certificate.json
.venv/bin/python scripts/verify_cayley_spectral.py model.mps.gz spectral-certificate.json
```

この検査器は元行から競合を再構成し、座標が全頂点の置換であること、対角重み0、非辺の重みが非正であることを確認する。RustのFWHTとは別の直接整数Walsh和で全固有値を求め、DとLからfloor(n(-L)/(D-L))を再計算する。LP状態や許容誤差を境界証明に使わない。点の選択数とこの元グラフ全体の上界が一致すれば、探索木を再生せずに最適性を確認できる。

`benchmark.py`のsnapshotは`scripts/*.py`と`src/**/*.rs`を全コピーするため、この新検査器も隔離した評価へ含まれる。最終snapshotでは全payloadのSHAを再照合する。独立スペクトル証明がない一般の探索完了は、内部探索木まで独立再生したと主張しない。
