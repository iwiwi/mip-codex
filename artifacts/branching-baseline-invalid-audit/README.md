# GLPK保存解の監査

native最適化は再実行していない。元MPSのhashはoriginal-summary.json参照。solution.rawは稼働snapshotのGLPKネイティブ出力のコピー、solution.solは独立検査用。検証は全て最適化を行わない。

```
cc -O2 glpk-native-kkt.c -lglpk -o /tmp/mip-glpk-check
/tmp/mip-glpk-check data/raw/benchmark/cbs-cta.mps.gz solution.raw
```

GLPK 5.0のglp_read_mps、glp_read_mip、glp_check_kkt (GLP_MIP、PE/PB) を使う比較専用プログラム。RustソルバーはGLPKを呼ばない。GLPK自身もrow5862のupper0/activity1.44935807712805、PB違反1.44935807712805を報告した。公式GMP相対検査と新GMP絶対検査も不合格で、native optimalは検証済最適として採用しない。
