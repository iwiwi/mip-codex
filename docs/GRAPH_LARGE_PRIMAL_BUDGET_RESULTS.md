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
