# 対角基底から始まるprimal探索の動的edgeノルム価格付け

main PR94/FP129f1ca68d72b01eから別branch。根LPのPhase IでDantzig価格付けのpivot数が大きいため、対角基底から始まる通常primal runだけ、edgeの二乗ノルムを追跡してreduced/sqrt(gamma)で順位付けする。既存のzero-cost適応L1（NORMALIZED）とdual simplexは変更しない。MIP_LP_EDGE_PRICING既定off、CLIは0/1だけ受理する。

初期はgamma_j=1+||D^-1 a_j||²。pivot列d=B^-1 a_enterと行rに対し、w=B^-T e_r、t=B^-T dを追加で解く。各非基底qのalpha=(a_q^T w)/d_r、cross=a_q^T tからgamma'_q=gamma_q−2 alpha cross+alpha²(1+||d||²)、出る変数は(1+||d||²)/d_r²へ更新する。浮動誤差で1未満は1へ抑え、非有限値なら旧価格付けへ戻す。primal restorationで基底が任意に変わった場合も戻す。基底が対角でない/eta更新が残るwarm runは対象外。比率検定、Bland退化処理、最終certificate、区間認証、許容差、deadline/pivot上限は変更しない。

ノルムは候補順位にのみ使う。2回の追加BTRANと全列内積が必要で、pivotが減っても遅くなる危険を含む。全Rust/生成LPとHiGHS oracle/元MPS根点のFraction検査/大きい開発根LPの10/60秒対照で判定し、単一モデル名で発動条件を調整しない。既存部分価格付けはgermanrr悪化で不採用だった。成分主解修復の採用審査とはsourceを分ける。全体優位・新記録は未達。

初回編集はCLI環境変数の行名不一致でその部分だけassert停止、続いたビルドも子探索のStats再集約3項目不足で失敗した。ログを保存し、元Resultの対応カウンタを合算するように修正して再実行する。

全245Rustはflag0/1で両方成功。元MPS生成LPはordinary200/scaled100/units199照合成功、units114はnumericで未解決。保存した同入力のflag0/1を再走して双方numeric（96/59pivot、後者53edge更新）を確認し、scaled_post_validationでpoint_valid=false、bad_rows=[]が両側に出た。499照合成功+1両側未解決として扱い、500成功とは書かない。
FP40677d6724ad1904固定。germanrr根LP60秒は同下界45980135.41639878、7923→7005pivotだが19.520575962→24.932506486秒（1.277倍）へ悪化、edge calls1/更新1992。neos-860300は従来14.985954059秒/22190pivot/下界1666.0685857415328から、有効側60.000752925秒/48043pivot/点・下界なし（edge更新48043）へ悪化。実例2件が悪化したため本案は不採用。188MIPへ広げず、根点3出力の元MPS Fraction検査とnegative evidenceを残す。

保存した根LP点はgermanrrの前後とneos-860300従来側の3出力で、元MPS Fractionの行/境界/目的検証が全有効（整数性はLPとして緩和）。有効側neosは点なしを保持。生成LPは既存driverにより生成器/seed/各照合ログと失敗入力を保持し、成功500入力の全MPS/全JSONを保存したとは主張しない。500profile出力のうち499照合/1両側未解決であり、スケールを変えた同系問題を500独立問題とは数えない。
