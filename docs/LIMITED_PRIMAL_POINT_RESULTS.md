# LP制限時の元問題実行可能点の回収

main PR108との組み合わせを評価した。LPが時間/iteration制限に達しても、残った実行可能点を元の変数単位へ復元し検査する。整数性を満たして元MIPの行・上下限・目的を通す点だけをincumbent候補にする。LPの終了状態は制限のまま、最適・不能へ変換せず、下界を追加しない。既定有効、MIP_LP_LIMIT_POINT=0で解除できる。

開発188問各10秒の設定切替比較は主解1改善・0悪化・187同等、下界1改善・1悪化・174同等・12比較不能。有限下界は前176・後176問。376出力の証明と123点が独立有効。終了状態は{"before": {"time_limit": 171, "resource_limit": 11, "optimal": 1, "numerical_error": 5}, "after": {"time_limit": 171, "resource_limit": 11, "optimal": 1, "numerical_error": 5}}。下界の変動は既存の時限処理によるもので、この機能の改善とは数えない。

supportcase42は60秒でも回収無効では主解なし/下界0、有効では主解40/下界0、回収点1・採用1。native10秒は自作40/0、HiGHS40/0、SCIP8/7.3440703963、GLPK主解なし/7.3435396106（主解/下界）。両測定4点・自作3証明は独立有効。総合優位や新記録の達成ではない。

全Rust282成功・4無視。100個の独立LPを各6上限・warm/cold・2設定で照合する2400状態はすべて有効で、制限時の追加点163件。各点の元LPをFractionで絶対1e-6検査し、最適値は比較専用HiGHSと照合した。min/max・目的offset・pivots0のMIP試験は主解を回収しても状態iteration_limit・下界なしを確認する。

最終指紋2b4fde253012fd32は通常ELF・ライブラリ・helperの出自が一致。点はGMP絶対1e-6（行/境界/整数性）、目的は相対1e-8で元MPSへ照合し、元モデルの証明はFractionで別途検証した。許容幅を緩めていない。旧親・セットアップ失敗もlimited-research-evaluationに、PR107/108併用の全記録をlimited-final-evaluationに保持する。保留52問は採否に使用していない。
