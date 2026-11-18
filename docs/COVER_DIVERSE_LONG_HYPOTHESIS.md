# 自作同値親からの長時間weighted探索（私的）

既存heap-profile（SHA6675d4a0a3ecd4bc39f75a50b4483075386b6610c450a9bdb9e76bac85ac767d）の変更なしで、自作neutral親17点から3点を選ぶ。parent0に対し、選択済み点への最小symmetric differenceが最大の点を順に選択する（同値時小index）。選択は[0,16,7]、相互距離924/820/848。全点自作1197点で、native/掲載解は使わない。

CPU40/47/50にそれぞれ固定、16GiB/1thread、単round100M反復上限・900秒guard・single、1020秒process-group watchdog。探索前の親を元GMPで再検査し、全出力も元GMPで検査する。目標/停止値は渡さない。900秒を使い切るとは限らず、iteration_limitで先に止まる結果も保持する。私的成績をmainに帰属せず、原問題のbound/最適性を主張しない。初期0.5秒の既存局所探索は時間依存である。

既存900秒継続・制限LNS・同値親matchingで1197を超えなかったため、異なる自作同値点からの軌跡を試す。品質改善が無ければその不改善も保持。新Rust source/外部optimizerへの委譲は無い。
