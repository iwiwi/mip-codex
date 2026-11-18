# 大規模で疎なcoverの同順位heap・本体検証

元sourcea45cdee97c/FPbdd9cc3a704b3b63（既定OFF）を凍結。main127へsource/tests/scripts/Cargo/build全byte同一で載せ替えたe1cdd817e4を経て、既定boolだけをONにしたa6c196e155/FP9c9eb50415d5914cを別凍結した。旧public APIはscanを維持するwrapper、main CLIはMIP_GRAPH_COVER_HEAP=0/1で比較する。

適用はn>4096かつ8Eceil(log2 n)<=n²、144n+32E<=memory。満たさなければheap配列を作らず旧scan。score降順/age昇順/id昇順の厳密な順序を維持し、100 random graphs×300step×移動/重み変更後に全score/未被覆辺/heap先頭/全position/親子順を再計算照合。頂点4999の辺・heap位置、gateのサイズ/密度/追加メモリも検査。初期と最終は各353Rustpassed/4ignored、各200small/1200出力の全列挙・元GMP・strict proof/7410tampers。

12大規模random-core出力は初期/最終それぞれ全元GMP/strict proof、各component全列挙からbest<=alpha<=upperを確認、実heap6出力。初期は6ペア中2件で主解+1、他4件同じ。局所fractionの時間打切りで上界が両方向に動いた値も保持する。原問題の誤optimal/infeasible主張はない。

初期188問はprimal186same/1up/1down、dual173same/2up/0down/13inc、finite175ずつ、statuses両182time/2optimal/4numeric。全376fresh proof/128点GMP＋公式valid。最終188はprimal187same/0up/1down、dual173same/2up/0down/13inc、同finite/status、同376fresh/128GMP公式valid。両者とも実heap適用0、改善を本機能へ帰属しない。neos-911970のprimal悪化は両screenに残し、最終9c9同一binary/同CPU46の3交互順pairsを追加した。6点全GMP/proof valid、3pairsすべて両primal140.59/dual23.259999999973406で同値。元188の悪化は保存したまま、機能不適用の変動との整合性を観測し、因果は断定しない。

本体z26/60秒は初期も最終もprimal両−1154。初期scan83,966/heap316,405反復、2.4006/2.4006秒で実heap_usedがfalse/true。最終はdual−1575/−1578も保存し、境界の改善は主張しない。初期300秒ではscan−1160/LB−1578→heap−1178/LB−1575、最終300秒ではscan−1163/LB−1575→heap−1182/LB−1578。計算予算/反復上限/再始動は変更しない。同一CPU44/1thread/16GiB、全主制御7+長制御2の9点は初期・最終それぞれ元GMP/strict proof valid。改善するprimalと変動するdualを両方報告する。

cod105は両opt−12/LP1/13pivots/約.0209秒で新経路非適用、S3は両opt−16/6042153nodes、最終19.381097/19.321179秒。生成5000pairはrootopt−2500。native60秒資料は親PR127と同入力/比較driver/validator SHAのまま参照し、300秒の自作比較と同一時間のnative比較として混同しない。正式240は未起動、holdout個別成績を調整へ使わない。

private prototypeの固定100k×12pairs24点は同values/同weighted counters、全GMP。疎5000pair約16x、z26約3.6x、S7約1.13x、密cod105は5x遅かったため一律適用を棄却し、上記構造gateを採用候補とした。private1197点を本体へ帰属せず、新記録/全体優位は未達。別private mean64/親再結合/LNSの不改善も保存する。

最終主制御7点と長制御2点、最終188の全376fresh/128元GMP公式、12large actualheap6もすべて通過。既定ONの開発376jobはbefore明示0/after未設定、large6pairsも同じ条件をjob.envで確認する。native300秒のHS/SCIP/GLPKを同CPU46/16GiB1threadで追加実行中。比較が完了してから採用コメントへ追記する。heapの固定順位・追加メモリguardと小/大生成例の正しさ、本体の同時間主解改善を採用根拠とし、非適用neos悪化と300秒dual3低下も隠さない。
