# 大規模packing容量拡張・初期候補の結果

候補core580d517295/f293、既定OFF、同一凍結CLIで容量flag OFF/ON。main126/E5は未変更。private全5点と元MPSは別ブランチlarge-packing-profileに保管。本体候補の348Rust/4ignored、1200small exhaustive/GMP/proof、7410tampers、40large analytic/GMP/proof、160tampers、主制御7点GMP/proofが通過。全40largeの実出力は0nodesだが未使用env MIP_NODE_LIMITを設定していたため事前node0設定の実験とは主張しない。

開発188問の10秒比較、全376新規strict proof/128元点GMPと公式検査通過。primal187same/1up/0down、dual173same/1up/1down/13incomparable、finite175ずつ、statuses両182time/2optimal/4numeric。新しい>4096graph認識は0。blp-ic98 primal改善、fast0507 dual悪化、seymour1 dual改善は新feature認識に対応しない観測であり、悪化も全保存する。タイミングの影響は推論で、機能の効果と断定しない。

本体z26/60秒はOFF primal−7/LB−1651/65nodes→ON−1154/LB−1575/73143nodes、すべて元点GMP・strict original Fraction cover/root/prefix証明valid。ON weighted cover2.400519秒/83638反復、root上界1575、independent_optimality false。比較用CPU46/同16GiB1thread/60秒はHiGHS1.16−1147/LB−2114、SCIP10.0.2−1013/LB−1650.9375、GLPK5は60.903269秒で解/境界なし。前2点GMP valid。外部点を自作探索に渡していない。z26は240外、掲載best未到達、新記録や全体優位の主張なし。

cod105の両flagはopt−12/LP1/13pivots/約.0209秒、S3は両opt−16/6042153nodes/19.39956 vs19.41236秒。生成5000pair本体はopt−2500/0nodes/.05315秒、厳密root最適性と元GMPもvalid。

最終候補では大規模再帰の資源上限も明示する。>4096頂点の探索だけ深さ512到達時にdepth_limitで中断し、元の安全な根/prefix境界と候補解を返す。多数の孤立点＋小さなoddcycleで深い再帰を生じさせる生成例を追加し、誤optimalを返さず中断することを確認する。既存<=4096は無変更。既定ONの最終ソースで全tests・生成例・188問比較を再実行し、採否を判断する。初回ENOSPC/fulltest/管理停止と無再探索復旧はJOURNALおよび台帳に保存。
