# 大規模packing容量拡張・初期候補の結果

候補core580d517295/f293、既定OFF、同一凍結CLIで容量flag OFF/ON。main126/E5は未変更。private全5点と元MPSは別ブランチlarge-packing-profileに保管。本体候補の348Rust/4ignored、1200small exhaustive/GMP/proof、7410tampers、40large analytic/GMP/proof、160tampers、主制御7点GMP/proofが通過。全40largeの実出力は0nodesだが未使用env MIP_NODE_LIMITを設定していたため事前node0設定の実験とは主張しない。

開発188問の10秒比較、全376新規strict proof/128元点GMPと公式検査通過。primal187same/1up/0down、dual173same/1up/1down/13incomparable、finite175ずつ、statuses両182time/2optimal/4numeric。新しい>4096graph認識は0。blp-ic98 primal改善、fast0507 dual悪化、seymour1 dual改善は新feature認識に対応しない観測であり、悪化も全保存する。タイミングの影響は推論で、機能の効果と断定しない。

本体z26/60秒はOFF primal−7/LB−1651/65nodes→ON−1154/LB−1575/73143nodes、すべて元点GMP・strict original Fraction cover/root/prefix証明valid。ON weighted cover2.400519秒/83638反復、root上界1575、independent_optimality false。比較用CPU46/同16GiB1thread/60秒はHiGHS1.16−1147/LB−2114、SCIP10.0.2−1013/LB−1650.9375、GLPK5は60.903269秒で解/境界なし。前2点GMP valid。外部点を自作探索に渡していない。z26は240外、掲載best未到達、新記録や全体優位の主張なし。

cod105の両flagはopt−12/LP1/13pivots/約.0209秒、S3は両opt−16/6042153nodes/19.39956 vs19.41236秒。生成5000pair本体はopt−2500/0nodes/.05315秒、厳密root最適性と元GMPもvalid。

最終候補では大規模再帰の資源上限も明示する。>4096頂点の探索だけ深さ512到達時にdepth_limitで中断し、元の安全な根/prefix境界と候補解を返す。多数の孤立点＋小さなoddcycleで深い再帰を生じさせる生成例を追加し、誤optimalを返さず中断することを確認する。既存<=4096は無変更。既定ONの最終ソースで全tests・生成例・188問比較を再実行し、採否を判断する。初回ENOSPC/fulltest/管理停止と無再探索復旧はJOURNALおよび台帳に保存。

## 最終候補と採用判断

1572f4382c/FPabc0956a38a27666、既定ONで350Rustpassed/4ignored、1200small/GMP/proof/7410tampers、40large解析最適/GMP/proof/160tampers、深さ制御4点GMP/proof/2tampers、CLI不正4文字列の拒否を確認。最初の深さ生成例は実際には2nodesでoptimalを証明し、テスト期待が誤っていた。optimizerは変更せず、その正しい早期証明も残して多数C5例を追加した。初回log/sourceを保存。depthdriverのスペース欠落もpy_compile検出時点で保存しdriverのみ修正。

最終188問: primal185same/0up/3down、dual173same/0up/2down/13incomparable、finite175ずつ、statuses両182time/2optimal/4numeric。全376新規proof/128点の元GMP＋公式valid。大容量新認識0。初期188結果と区別し、map10・neos-911970・supportcase12のprimal悪化、seymour1・sp98arのdual悪化を全保存。

同一凍結ABC、CPU46、各10秒×3交互順ペア×5件の30点が全GMP/proof valid、全graph None。15pairsでprimal2up/1down/12same、dual2up/1down/12same。map10は−24/−25、残2pairsは両−25。neos911970は129.38/140.59、111.47/111.47、129.38/111.47。supportcase12は全両−12。seymour dualは同値/改善/悪化、sp98arは同値/同値/改善。これを元188の悪化の代わりにはしない。非適用経路の時間打切り変動と整合するという推論であり、因果を断定しない。

最終主制御は7GMP/7strict valid、z26/60秒OFF−7/LB−1651→ON−1154/LB−1578、depth_hit false。初期−1575とは区別、nativeは同一入力/driver/validator SHAを再照合して初期60秒比較を参照する。原型private1190や別のheap私的結果をこの容量変更のmain成績へ帰属しない。

判断: 採用。厳密な構造・作業量・メモリguardを保って未知の大規模packingも自作Rust経路へ入れ、元点と安全な境界を検査できた。新領域z26で本体の主解/境界が改善し、生成例・全列挙・型/再帰資源検査・全376監査を通過した。開発188の悪化は保留/隠蔽せず、その値と追加の両方向変動をすべてPRへ報告して判断する。全体優位・新MIPLIB記録は未達、正式240は未起動。外部ディスク障害と時間が重なるnative1件を同一凍結版・資源で再測定する事前固定方針はBASELINE_DISK_RECOVERY_20261118.mdに分離し、元720を保持する。
