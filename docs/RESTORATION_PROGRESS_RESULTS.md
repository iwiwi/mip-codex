# 進捗があるPhase Iの実行可能性復元

main45257f737/FP3365のrestore_primalは、LP全体のstats.restoration_attemptsが4以上なら常に数値失敗とする。niddaでは各回復の間にPhase I目的値が2705→2383→1568→1189→481→…と減少していた。単に回数を8へ増やす旧案は足りず、失敗時の線形方程式残差補正・Phase I価格付け再始動も点を得なかった。

本試行では各primal run内で、前の回復時点より目的値が1e-7*(1+abs(previous))以上減少した場合だけ連続回復回数の基点を現在の総回数へ進める。進展しない回復は4回まで。Phase I/IIは別の費用を持つため基点も別。statsの総回数、総pivot/flip、deadlineは返さない。元の許容差・最適/実行不能証明と原問題検査は維持する。問題名や既知値を条件にしない。

初版10秒純LP診断ではniddaが79 pivotsの数値失敗から431 pivots/約0.023秒でLP点を得た。13回の復元を使った。元MPSのFractionでbounds/rows/目的値を照合し有効。行違反約2.88e-11、目的値差約4.92e-11。自作LP bound−1216923.270104406に対し同CPU0 HiGHS LPは−1216923.2701044157（約9.8e-9差）。整数解ではない。ranは3476 pivots/3444.42106643829を維持。app1の根も4065 pivots/−22.824355359110097を維持。kumeu/uccase9は10秒時点で点なしのまま数値終了→時間制限に変化するので改善と数えない。neos860300は10秒で両方根未完了。

採用は未定。総予算を返さないテスト、既存unit/2700合成/根LP oracle/数値回帰、元MPSの全出力点検査、development188の10秒比較、長時間のneos860300/germanrr等の回帰、kumeu/uccase9の60秒診断とniddaの実MIP・native比較を実施して判断する。別branchの3行集約はまだ未採用であり本試行へ混ぜない。

初版の根診断保存点はnidda新点+ran/app1前後の5点がFraction有効。初版source FPc4f3e18411e5f157を保存した。後のcargo testが通常release binaryも更新したため、初版ELFは上書きされた。初版sourceは後から追加した統計/テストだけを除いて再構成し、測定時FPとの完全一致を確認した。以後は正式測定前にELFとsourceをまとめて凍結する。

累積4回を超える新規処理数をlp_restoration_extended_attemptsで分離した。新テストは、進捗基点を進めても既消費pivotとdeadlineが残ることを確認する。最初はlimit=8で7消費済・1pivot後に直ちに証明を返すと誤って期待しIterationLimitで失敗した。実装が次の反復前に予算を確認する挙動を保ち、成功例をlimit=9へ修正した。失敗ログは保持する。

統計/予算テスト追加後のcore703e3b717、FP8b1969fa725c8ce2をrestoration-progress-final-diagnostic-20261116へ凍結した。211 Rust/2700合成/根300設定500 engine出力/数値32件（18有効点・14精度未解決）が通過。数値32件の全点が有効になったとは述べない。

現行版のnidda10秒MIPは14054.945596895の主解を約0.036秒で得たが、約1.56秒・70nodeでnumerical_errorとなった。従来は根で数値停止し点なし。現行版も完走/最適証明ではない。同CPU40/10秒nativeの実run smoke-20261116-072956-4yi5ftokはHS14092.353218828/−966442.21472557、SCIP13848.451257457/−585853.86821694、GLPK13926.554167189/−528527.06098571。自作の根境界は−1216923.270104406で全nativeより弱く、主解のみHSより良い。nidda新点、ran前後2点、native3点の合計6整数点が元MPSのGMP絶対許容差/公式checkerとも有効。

kumeu60秒は約48.46秒・39372 pivotsで数値停止し点なし。uccase9も約60秒・14169 pivotsで点なしの時間制限。この2件を性能改善として数えない。neos860300/germanrrの60秒MIP回帰と188開発問題10秒の比較/独立監査は継続中。188比較はmainFP3365と本凍結版をCPU0..2の3 shard、1thread/16GiB ASで交互順に実行する。3行集約は両方無効。原結果はartifacts/restoration-progress-development-*へ保存。

棄却した双対逆積補正・primal方向補正・数値失敗後PhaseI再始動の実データはartifacts/lp-recovery-negative-evaluation（680 files/4,307,049 bytes）へSHA relocation ledger付きで保存した。3 ELFはlocalに保持しhashを記録。各凍結版のsrc/Cargo/build fingerprintはELFのbuild metadataと一致する。初版進捗案のELFだけは後続cargo testに上書きされたが、測定時FP一致のsourceを再構成保存し、その制限を明記した。

## 全188対完了と採用判断

10秒・1thread・16GiB AS・開発188問題の前後比較が完了。主解1改善（nidda）/0悪化/187同値、dual0改善/1悪化（loue）/77同値/110比較不能。before169 time_limit・8 numerical_error・11 resource_limit、after172・5・11。点なしで数値停止→時間制限になったものを主解改善に数えない。376出力のカット証明は全件Fraction有効、保存整数点109点は全件元MPSのGMP絶対許容差/公式checker有効。最適宣言は前後0件、未解決10秒ペナルティのshifted geometric meanは両方10秒。

loueのdual22966→22953は、回復処理が両方0回で、根cut96→64本が残る時間境界の差を伴う。実際の根cut予算はmin(時間5%,0.25秒)であり0.5秒ではない。60秒も同上限のため差は残った。strong offでも旧22962/新22953。新状態をSimplexへ置き回復メソッドをinlineしない対照版（FP087efc4b6420f92e）はloue22957に留まり差を解消せず、nidda/nu25の主解も元案と同じだった。対照版211 Rustと6出力/3整数点の独立検査が通過したが、複雑化の対価となる改善を確認できず未採用。元の簡単なFP8b版を採用候補とし、小幅のdual悪化を隠さない。

非発動binkarの純LP12交互対では中央値0.078620602→0.0785486005秒（比0.999084）。時間/追加ゼロ統計を除く共通JSONは全12対でbit一致。全LPで速度改善したとは主張しない。

主workspaceでFP8b1969fa725c8ce2を再buildし、同CPU41・10秒・1thread・16GiBでrust/HS/SCIP/GLPKを比較した実run smoke-20261116-075942-fm60z3ifは、rust14054.945596895/−1216923.270104406/numerical_error、HS14092.353218828/−966442.21472557/time limit、SCIP13848.451257457/−587469.85377837/timelimit、GLPK13926.554167189/−528527.06098571/feasible_limit。自作の主解はHSだけを上回り、dualは全3者より弱い。同workspaceのbinkar再現run smoke-20261116-080014-luf774m2は6888.550001/6646.313678834を維持。両run合計5点がGMP/公式とも有効。主workspaceのfmt検査も通過。

全240/3600秒/1thread/16GiBのsnapshot restoration-progress-final-20261116を作成し全hashを照合したが、全件実行は未開始。experiments/restoration-progress-snapshot-20261116.jsonに記録。最終全件は11月19日UTCを目安に凍結・開始し、11月22日14:59 UTC期限を守る。native全体優位・世界記録は未達成。
