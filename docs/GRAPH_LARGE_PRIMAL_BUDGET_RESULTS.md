# 大規模疎packing予算候補の途中結果

core515fd3555b、FP e180c8c80bbfc8ca、凍結CLI SHA72ae82b1027095bb5bc00343fc1719a498fc225a093585574cf6e096612cc51a。defaultOFF、既存heap operation/memory gateを満たす場合のみflag1で5%→25%。既知解・問題名・外部点は探索に使わない。

Rust353passed/4ignored。小規模200×6=1200出力がGMP/全列挙/strict proof成功、7410改ざん拒否。large random-core6×2=12出力は全GMP/proof、component全列挙alphaを安全に挟む。全12実heap使用。主解6組は同じで、upperは2同値/4悪化（+9,+1,+1,+1）。初回検査器path未発見1200未検証と生成driverの旧heap flag assert失敗を含む全raw証拠をcorrectness archiveに保持。candidate_source_changes=true、canonical_main_changed=false。

CPU44・1thread・16GiB、同一凍結CLI、flag明示0/1。全9出力が元MPS GMP/strict proof valid。

| 入力/秒 | OFF primal/dual | ON primal/dual | 結果 |
|---|---|---|---|
| generated5000pair/10 | 未測定 | −2500/−2500 | root最適 |
| cod105/10 | −12/−12 | −12/−12 | 両root最適、0nodes |
| sorrell3/30 | −16/−16 | −16/−16 | 両最適、6,042,153nodes、19.377908/19.351659秒 |
| z26/60 | −1154/−1575 | −1178/−1578 | primal24改善、dual3悪化 |
| cdc7-4-3-2/60 | −279/−438 | −279/−438 | 同値 |

z26 weighted反復は296,501→2,428,697、CDC61,301→430,738、weighted時間2.401598→14.401824秒だがCDC主解は増えなかった。z26/CDCは評価240問外、cod105/S3は開発用。5,000pairのOFFは以前の容量/heap検査で成功しているが、この候補ではON1点だけ測定したため新しいペア比較とは数えない。

300秒比較は実CPU45に固定。copied metadataのcpu44誤記は訂正receiptに保存してrawを保持。188件開発screenと376出力の独立GMP/公式検査は進行中。holdout52は参照しない。採否は未決、main128と未起動最終snapshot9e5は維持。新記録・全体優位とも未達。

## z26/300秒の先行完了

同一凍結CLI・実CPU45のOFF/ONは−1181/LB−1578（300.041123989秒）→−1188/LB−1575（300.040943413秒）、全元GMP/strict proof valid。主解7点改善、下界も3点改善。60秒の下界悪化も取り消さず保持。

同日PR128での元native300秒（CPU46・同マシン・16GiB/1thread）のraw証拠も別folderへコピー。HiGHS1.16.0−1147/LB−2114、SCIP10.0.2−1013/LB−1539.949499925362、両GMP valid。自作候補は両者より主解が良いがSCIPの下界が強い。GLPK5は420秒watchdogで時間内点なし、wrapper timeout後の441.545秒で遅着した−460/LB−1651は別GMP validだが時間同条件の品質比較から除外。正式cgroup測定とは別の研究driverのtimeout欠陥で、全失敗/遅着rawを保持した。その教訓から新CDC native300driverはprocess group全体killを用いる。

## 開発188問完了（defaultOFF源515fd、同一CLIの明示0/1）

376出力のstrict proof全成功、可行点128件すべてGMP/公式検査成功（376fresh、再利用0）。新予算の実発動0。primal187同値/1悪化、dual172同値/3改善/13比較不能、finite dual各175。両status内訳time_limit182/optimal2/numerical_error4。元の失敗も含め全188問を保存。時間のshifted geomean（shift1/未解決10秒）は9.724145755/9.724146697秒で実質同じ。

変化はfast0507下界165.969633434→166.079437576、leo1下界389771802.153→389874005.625（両点なし）、seymour1下界394.872509052→394.992394258、supportcase12主解−13→−12（下界両−537783.5007328897）。新機能の未発動路であるため改善・悪化を新機能の数学的効果と主張しない。supportcase12を同CPU44/交互順3pairsで別途再確認中だが、元188損失を削除・置換しない。


## 初期候補の採用方向とdefault検証へ

CDC300秒もOFF−279/LB−434（300.004574448秒）→ON−285/LB−429（300.004397936秒）、両GMP/strict proof valid。2つの評価外純packingで300秒主解が改善し、現188の新配分発動0・全376証明/128点GMP公式成功を確認したため、候補のdefaultをONへ変えて再ビルド・全検査する。未統合であり、main128/9e5は維持。生成4上界悪化・z26/60秒下界悪化・未発動supportcase12損失も採用判断で保持する。

supportcase12追加3交互pairsは両−13/LB−537783.5007328897の6点、全GMP/proof成功。元188の−13→−12は削除/置換しない。


## CDC native300秒の完了

CPU46/16GiB/1thread、HiGHS1.16.0は−280/LB−1111（306.7299885749817秒）、SCIP10.0.2は−237/LB−11744（300.005887秒）、両GMP valid。時間超過も保持。GLPK5は420.007275108秒でprocess group watchdog、rc−9、時間内出力点なし。群全体kill後、対応するlive processなしを別receiptに保存。初期候補ON−285/LB−429は両者より主解・下界が強いが、単一評価外入力の研究結果で、全体優位ではない。defaultON源の再測定は別途進行中。
