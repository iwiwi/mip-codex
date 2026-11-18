# 大規模疎packingのweighted長round仮説

base main129/3a774900/FP602。新flag `MIP_GRAPH_COVER_LONG_ROUNDS` は初期defaultOFF。既存heapが実際に適用される大規模疎構造と64bit usizeの場合だけ、改善後の単round上限を2M→100Mにする。最初の未改善時上限500K等は維持し、改善したroundの後に行う既存最大8再始動では、長modeなら次roundの最低反復予算も100Mにする。全体/局所期限が先なら即停止。乱数式・重み・forgetting・状態遷移・厳密点検査・探索木/境界・memory guardは変えない。旧公開APIは短roundのwrapperを保持。Stats.long_rounds_usedは実heap構築後にmodeが適用されたことを示す。

根拠: 予算ONのz26/300秒では割当75秒に対し旧weightedは8M反復44.43秒で停止、主解1188。私的な無改変長roundは自作同値親から100M反復で1198に到達しGMP/公式検査成功、掲載bestと同着だった。本体ではこの親を読み込まず、元入力から汎用探索を行う。私的点・問題名・既知目的値はsolverに埋め込まず停止値にも使わない。

初期/最終各353テスト、1200全列挙/GMP/proofとtamper、large disconnected coreの厳密oracle、開発188/独立監査、評価外z26/CDCの60/300/必要なら900秒で比較する。全損失/誤答/未検証を保持し、主解改善だけで最適性を宣言しない。点/証明は元MPSへ独立検査し、比較nativeの点は与えない。正式自作240は未起動のまま、holdout個別結果は使わない。

100Mは状態を100M回安全に反復する上限で、新しい数値許容は導入しない。実heapのn≤65535からE≤n(n−1)/2。各辺重みは初期1から1stepにつき最大1だけ増え、forgettingでは縮むため≤100M+1。scoreの保守的な一時幅2(n−1)(100M+1)はi64内、全重みとその2倍はu64内、9roundのweight-increase総計9E100Mもu64/64bit usize内。forgettingの3倍中間値もu32内。独立Python整数計算をexperimentsに保存する。32bitでは新modeを適用しない。

初期188問比較完了: 376 fresh proof/128点fresh GMP公式valid。primal185同等/3改善、dual174同等/1悪化/13比較不能。新mode使用0件のため改善を新機能に帰属しない。n3div36 dual111675.17133008162→111478.90476012451の悪化も保持。300秒z26 primal−1188→−1193/dual−1575→−1578、CDC primal−285同等/dual−442→−438。全13controls元GMP/proof valid。1時間比較は実行中で採用判断は保留。

1時間比較の前半とnative3完了: 同一CLI3222/CPU46のOFFは−1188/LB−1578、3,491,073nodes、3600.042092849秒、weighted8M反復46.044641313秒/3restarts、元GMP/proof valid。HS1.16.0(CPU51)は−1147/LB−2114/3600.046329737秒、SCIP10.0.2(CPU52)は−1070/LB−1533.4885270508491/3600.006063秒、両点元GMP valid。自作OFFの主解は良いがSCIPの下界が強い。GLPK5(CPU53)は3660.012435156秒のwhole-process-group watchdog、rc−9、時間内受理点なし、該当group生存processなしを確認。超過失敗は保持。全1thread/16GiB、外部点はRustへ渡さない。ONは2026-11-18 11:05:57UTC開始で実行中。

初期一時間比較完了:同一frozen3222/CPU46、OFF−1188/LB−1578/3600.042092849秒→ON−1196/LB−1575/3600.039068732秒。両元GMP/strict proof valid。ONはweighted189054109反復899.399733885秒/1restart、最終改善40498676step。HS−1147/LB−2114/3600.046330秒、SCIP−1070/LB−1533.488527/3600.006063秒（両GMP）、GLPK3660秒watchdog失敗を全保持。候補の主解は良いがSCIP下界が強い。新記録/main達成ではない。main131統合後既定ON core4d4b1b5e3aを再検証し原source50feと区別する。

合成後の最終300秒4出力は全元GMP/proof valid。z26 OFF−1187/LB−1578→ON−1193/LB−1578、CDC OFF−285/LB−434→ON−286/LB−424。最終188は376fresh proof/136点fresh GMP公式valid、primal186同等/1改善/1悪化、dual173同等/1改善/1悪化/13比較不能、両182time_limit/2optimal/4numerical_error、finite175。全認識graphはcod105/S3の1024varsで長mode発動0。mcsched314588→313589改善、kasai1072833.43→1072846.63悪化、seymour1下界改善、sp98ar下界悪化も全保持し成果に帰属しない。Kasai追加3pairs全1072833.43同等、sp98ar追加3pairsは各組同じbound（2組443248157.01279、1組444163912.977999）。12点GMP/proof valid、元損失を置換しない。最終large12は主解6組同じ、upper4同じ/1改善/1悪化、全oracle/GMP/proof valid。初期一時間の主解+8/下界+3と、最終z26300主解+6/CDC300主解+1下界+10を根拠に長round既定ON採用を提案する。全体優位/新記録は未達。
