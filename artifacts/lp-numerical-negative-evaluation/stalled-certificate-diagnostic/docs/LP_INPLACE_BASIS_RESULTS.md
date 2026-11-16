# 基底線形系の一時列コピー削減

2026-11-16、main 35f754becに対する独立変更。`Basis::forward/backward`では新しく所有したfaer Colを作った後、`solve/solve_transpose`がもう一列を複製していた。所有列へ`solve_in_place/solve_transpose_in_place`を適用し、その結果を返す。LU因子・eta更新・pivot/ratio選択・許容差は変更しない。外部最適化への委譲ではなく、既存の汎用線形代数APIの呼出し方だけを変更した。数学的操作は同じ三角線形系の解法である。

## 経緯と性能

未採用MIR初期版（別ブランチ879b2efe3、固定保存`artifacts/mir-initial-snapshot`）で、MIR offでもbinkarの根LPが約25%遅くなった。LP価格付けの未採用PR66にも同じ保存解悪化があったが、当時は原因を特定していない。MIR版では元の全LP反復数/出力が同じなのにperf stat命令数が約14%多く、短いサンプリングではfaer Col copyが33%を占めた。本変更はこの冗長コピーを削除する。LLVMの特定挙動まで原因を確定したとは扱わない。

旧版はfour-rounds-final-20261116のSHA検査済みバイナリを使用。CPU2でcut/heuristic全停止のbinkar根LP12組は時間以外の全JSON出力が完全一致し、中央値0.08662848→0.08468735秒（約2.24%減）。CPU3のperf stat各8回はcycles257.576M→252.347M、instructions1.26154B→1.24558B。ハードウェアカウンタはmultiplexされた実測であり普遍的な性能保証ではない。

大きい根LPの同一CPU比較も各2組、時間以外の全JSON出力が完全一致した（点の独立最適性再証明ではない）。germanrrはCPU6で18.6004→18.1929秒、18.1446→17.2108秒、全て7923pivot/下界45980135.41639878。neos-860300はCPU7で14.8846→14.6246秒、14.6959→14.5038秒、全て22190pivot/下界1666.0685857415328。これらはLP単体診断であり、nativeとの3600秒順位には換算しない。

CPU0/1、各10秒/1thread/16GiB、固定開発18問を旧新版交互に測定し、全18問のprimal/dualは許容差内で同じ。終了時節点数比の幾何平均1.01122（約1.12%増）。全てtime_limitであり、解けた問題数や総解決時間の改善とは主張しない。binkarは同CPUで保存解6888.550001を維持、2956→3002節点。pk1も19を維持、57956→60145節点。タイミング制限のheuristicにも影響するため、全問題の同一探索列は保証しない。

60秒/1thread/16GiB: binkar6865.969997/6646.31367883、timtab primalなし/120576、germanrr primalなし/45980135.41639878、neos3966/2031。既存のmainの到達値を維持している。既存native60秒比較ではbinkarはHiGHS/SCIP6742.200024最適、GLPK6755.150024/6716.384677、timtabはHiGHS/SCIP764772最適、GLPK1147266/417324.656023、neosはHiGHS/SCIP3201最適・GLPK4196/2123であり、自作がこれらを超えたとは主張しない。

## 検証と採用

202 Rustテスト、合成2700問（default500/binary500/larger300/mixed500/rounding500/projected400）、根LP普通100×2engine・scaled100×2engine・units100×revised、数値回帰32件を通過。数値回帰は10固定serialization点、22変換例（8検証済み点・14は従来どおり精度不足で未解決）であり、32個の実行可能解ではない。検査器8テスト通過。固定36出力と60秒4出力の全切断575項がFraction検証、28保存MIP点がGMP絶対検査と公式検査を通過。これは全B&B探索の独立リプレイではない。

原問題/原境界での独立検査を維持し、同じLP操作のコピーを減らす小さい改善として採用する。生記録・実際のsmokeディレクトリ・旧版と対の出力・SHA台帳は`artifacts/inplace-evaluation`へ保存する。自作240問full snapshotはcreate-onlyとし、今は開始しない。MIRはこの変更をrebaseして改めて比較し、初期版の性能退行を改善として混ぜない。価格付けの未採用判定も自動では変更しない。
