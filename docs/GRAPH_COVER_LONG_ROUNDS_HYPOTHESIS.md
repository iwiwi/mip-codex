# 大規模疎packingのweighted長round仮説

base main129/3a774900/FP602。新flag `MIP_GRAPH_COVER_LONG_ROUNDS` は初期defaultOFF。既存heapが実際に適用される大規模疎構造と64bit usizeの場合だけ、改善後の単round上限を2M→100Mにする。最初の未改善時上限500K等は維持し、改善したroundの後に行う既存最大8再始動では、長modeなら次roundの最低反復予算も100Mにする。全体/局所期限が先なら即停止。乱数式・重み・forgetting・状態遷移・厳密点検査・探索木/境界・memory guardは変えない。旧公開APIは短roundのwrapperを保持。Stats.long_rounds_usedは実heap構築後にmodeが適用されたことを示す。

根拠: 予算ONのz26/300秒では割当75秒に対し旧weightedは8M反復44.43秒で停止、主解1188。私的な無改変長roundは自作同値親から100M反復で1198に到達しGMP/公式検査成功、掲載bestと同着だった。本体ではこの親を読み込まず、元入力から汎用探索を行う。私的点・問題名・既知目的値はsolverに埋め込まず停止値にも使わない。

初期/最終各353テスト、1200全列挙/GMP/proofとtamper、large disconnected coreの厳密oracle、開発188/独立監査、評価外z26/CDCの60/300/必要なら900秒で比較する。全損失/誤答/未検証を保持し、主解改善だけで最適性を宣言しない。点/証明は元MPSへ独立検査し、比較nativeの点は与えない。正式自作240は未起動のまま、holdout個別結果は使わない。

100Mは状態を100M回安全に反復する上限で、新しい数値許容は導入しない。実heapのn≤65535からE≤n(n−1)/2。各辺重みは初期1から1stepにつき最大1だけ増え、forgettingでは縮むため≤100M+1。scoreの保守的な一時幅2(n−1)(100M+1)はi64内、全重みとその2倍はu64内、9roundのweight-increase総計9E100Mもu64/64bit usize内。forgettingの3倍中間値もu32内。独立Python整数計算をexperimentsに保存する。32bitでは新modeを適用しない。

初期188問比較完了: 376 fresh proof/128点fresh GMP公式valid。primal185同等/3改善、dual174同等/1悪化/13比較不能。新mode使用0件のため改善を新機能に帰属しない。n3div36 dual111675.17133008162→111478.90476012451の悪化も保持。300秒z26 primal−1188→−1193/dual−1575→−1578、CDC primal−285同等/dual−442→−438。全13controls元GMP/proof valid。1時間比較は実行中で採用判断は保留。
