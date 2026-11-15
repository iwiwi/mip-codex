# 根feasibility pumpの試作計画

PR20の目的変更warmを使い、主解なしの根だけで整数丸め点とLPへのL1射影を交互に行う。最初は既定offで独立対照する。主探索の証明・bound・枝刈りへ局所LPの結果を使わない。元Modelと元MPSの解検査を通った主解だけが改善となる。

二値/隣接二整数の変数はtargetに応じた±xの線形費用でL1距離を表せるため、元LPの基底を新費用で再利用する。その他の一般整数は距離変数d>=0とx-d<=target、-x-d<=-targetの2行を局所Modelへ加える。targetの変更はRHSだけ、行列は固定なので以後はwarm基底を使える。連続変数の費用は0。丸め先は根の整数境界へclampし、同じtargetへ戻れば固定seedで摂動する。未知/過大数値は安全に省略する。

対象はrevised engine、根、他の既定heuristics後も主解なし。時間予算は全体5%/最大.25秒、最大32LP/各4096pivotを初期値とする。必ず全体deadlineで止める。LP統計を合算し、iterations/cycles/perturbations/局所infeasible/数値失敗等を別途記録する。RINS/probeの内部では再帰無効。対象外・失敗・時間切れは元探索をそのまま続ける。

検証は純二値と一般整数のmin/max、連続変数との混合、局所infeasibleが親を枝刈りしないこと、time/node limit時のbound不変、固定targetの循環、元入力の解チェック。小1000/二値1000/中規模200、18開発問の0/1対照で判断。改善しない場合は任意設定のまま。holdout性能を調整に使わない。既存ソルバー呼出しは外部検証だけ。
