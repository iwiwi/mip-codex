# Phase IだけのDantzig継続

2026-11-16。main b0b4e946d / FP5a6ccff29edba788からの独立仮説。Phase Iの退化25回後にfirst eligibleへ切り替える現規則を、MIP_PHASE1_DANTZIG=1限定でDantzig継続にする。Phase IIは従来どおり。元目的0で既存のL1正規化スコアを使う経路では、そのスコアを継続する（全経路がraw Dantzigという意味ではない）。人工変数に正costがあるrunだけを対象とする。判定・許容誤差・pivot/flip/deadlineは変更しない。cycle等の非収束は予算終了し、不能/最適の証明とは区別する。既定OFF。

発端の行削除案は正しい21591証明を得たが、Neo860300根LP14秒→60秒未完了、murg主解2201→4027へ悪化して未採用。行削除OFFでPhase IだけDantzig継続すると根LP14.285秒/22190pivot→7.850秒/12157pivot、dual1666.0685857415328は同じ。germanrrは17.658秒/7923→17.387秒/8161、dual差6e−9。行削除＋Dantzigは5.743秒/17716pivotで完了したが、まずcore変更を独立に検査する。

元の複合試作はexact-row-dominance branch / phase1-dantzig-v1-diagnostic fd9e4eb13aa6ce09に保存。単独版を新たに凍結して選択MIP/LP、元MPS点検査、既存回帰を行う。改善前に既定採用しない。holdout入力は使わない。main全240 snapshotはvec-backed-rhs-final-20261116を作成/照合のみ、最終全件は未開始。11月19日UTC目安の全件開始と11月22日14:59UTC期限を維持。

独立初版4b36739d13762049の6問MIP10秒対照: Neo860300は下界なし→2002、binkarは6888.550001→6863.129996。murg2201、germanrr点/下界なし、nidda14054.945596895数値停止、app1点なし/−22.824355は同じ。既定flag有効の219 Rustを成功、複合試作からの保存5LP点も原MPS Fraction有効。

採用候補ではPhase I/IIをconst引数で分離し、Phase IIには設定読込や人工列走査を入れない。既存のL1正規化経路でもそのスコアを保つ意味を明確にし、flagをMIP_PHASE1_KEEP_PRICING=0/1（候補既定ON）へ変更。旧flagは凍結初版の再現専用。新sourceを凍結し同ELF0/1の188対照前に回帰・点監査を行う。
