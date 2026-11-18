# 同順位cover選択heapの私的検証結果

本体srcはmain126/E5不変。private source8d08262ad1。100random graphs×300変更で毎移動と重み変更後にheap先頭と線形走査の選択一致、全heap位置・親子順位、全score/未被覆辺/総重みを再計算照合した。高頂点番号4999のpairも検査。現ソースのexample unit tests6passed（旧5passedは別ログで二重加算しない）。元のmemory/ペア/期限/厳密原点guardは保持。

100k固定weighted反復、共通のGMP検査済み自作seed、CPU47、1thread、16GiB、120秒guard（全件iteration_limit終了）。各モデル3交互順ペア、全12ペアで最終valuesとweighted全カウンタ（seconds除く）が一致、全24出力の元MPS GMP valid、seed4点も再検査。初期0.5秒の探索カウンタは時間依存なので比較しない。選択の一致はunit testsで直接照合しており、fixed-work結果だけから全中間軌跡一致を追加主張しない。

| model | scan/heap weighted time比（3回の範囲） |
|---|---|
| pairs5000 | 15.7397–15.8630 |
| cod105 | 0.1955–0.2015 |
| sorrell7 | 1.1280–1.1355 |
| z26 | 3.5788–3.6268 |

比が1未満ならheapの方が遅い。cod105は約5倍遅く、全モデル一律適用は棄却。今後の本体候補は大規模・疎な構造をメモリ/operation見積りで自動判定し、密な小グラフでは旧scanを保持する。目的値・問題名では選択しない。S7の利得は約13%に留まる。z26は約3.6倍だが、まだprivate固定work結果でありmainの時間/成績には帰属しない。

次はz26共通seedからCPU47同一、交互順3ペア・各60秒・100M反復上限で品質を比較。既知解・目標/停止値は与えず、全点を元MPS GMPで検査する。

## 固定時間と長時間継続

同じ自作1190点seed・同CPU47・60秒を交互順3pairsで実行。3回ともscan−1190/heap−1193、全6出力元GMP valid。weighted反復はscan約2.4M/heap約11.6–11.8M。親seedと初期0.5秒が同じ目的値でも時間依存カウンタは一致を要求せず、fixed-workの厳密比較と区別する。

最良自作1193seedからheap300秒singleは−1197（59,281,900反復、最後の改善16,227,263）、reseed2M/roundは24round/46,385,332反復で−1193。両元GMP valid。1197seedから900秒/100M長roundの再始動2回は174,978,267反復で−1197のまま、元GMP valid。公式掲載best−1198に届かず、新記録ではない。すべてprivate componentで、main成績や原問題の最適性とはしない。

本体候補a45/BDDは別枝graph-cover-choice-heap。353Rust/4ignored、1200small/GMP/proof/7410tampers、12large random-core/GMP/proof（実heap6）を確認。本体z26/60秒は両−1154で反復83,966→316,405、300秒では−1160/LB−1578→−1178/LB−1575、両GMP/strict proof valid。本体の変化はこの別ソースの測定であり、private1197を本体へ帰属しない。
