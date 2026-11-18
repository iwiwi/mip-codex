# 自作同値親からの長時間weighted探索（私的）

既存heap-profile（SHA6675d4a0a3ecd4bc39f75a50b4483075386b6610c450a9bdb9e76bac85ac767d）の変更なしで、自作neutral親17点から3点を選ぶ。parent0に対し、選択済み点への最小symmetric differenceが最大の点を順に選択する（同値時小index）。選択は[0,16,7]、相互距離924/820/848。全点自作1197点で、native/掲載解は使わない。

CPU40/47/50にそれぞれ固定、16GiB/1thread、単round100M反復上限・900秒guard・single、1020秒process-group watchdog。探索前の親を元GMPで再検査し、全出力も元GMPで検査する。目標/停止値は渡さない。900秒を使い切るとは限らず、iteration_limitで先に止まる結果も保持する。私的成績をmainに帰属せず、原問題のbound/最適性を主張しない。初期0.5秒の既存局所探索は時間依存である。

既存900秒継続・制限LNS・同値親matchingで1197を超えなかったため、異なる自作同値点からの軌跡を試す。品質改善が無ければその不改善も保持。新Rust source/外部optimizerへの委譲は無い。


完了: 親[0,16,7]から100M反復各1回で−1197/−1198/−1197、component秒512.914195894/499.669962036/516.526172217。3点元GMP成功、親3点もfresh GMP。1198改善は親16から40,959,147反復目、最後も同反復、100M終了時点の点を保存。追加のstrict GMP/公式検査もvalid。公式ページをNov18再確認し掲載best−1198と同着、新記録でもmain達成でもない。追加監査の最初は出力directory未作成で書込み前に停止し、失敗ログを保存、directory作成後検査だけ再実行して成功。optimizerは再実行していない。

SOLは `solutions/exploratory/z26-1198-own-20261118.sol`、元入力gzip SHA63ca4683d1311d5e1390472d5dfb880af2a6100dd11a8a08006d90fa1e27b783。公式検査の再現例（checkerと元MPSを準備）:

```
third_party/miplib-checker/bin/solchecker data/raw/exploratory/z26.mps.gz solutions/exploratory/z26-1198-own-20261118.sol 1e-6 1e-6
```

より厳しい絶対GMP判定は保存result.jsonを `MIP_ABSOLUTE_CHECKER=.../solchecker-absolute .venv/bin/python scripts/benchmark_validate.py MODEL RESULT_JSON OFFICIAL_CHECKER OUTPUT_DIR` へ渡して再現する。OUTPUT_DIRは先に作成する。raw結果・双方の検証ログ/JSONとsource/親/入力SHAを全保持。次はこの自作1198から同値親生成を行い、全点再検査後に汎用的な距離選択で長探索する。既知値の停止判定は加えない。
