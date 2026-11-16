# 有限幅で正規化したdual行選択

親main PR96/FP084f7130a38dfe8f。対角基底からの動的edge価格付けは追加2BTRANと全列内積が重く、実例悪化で不採用だった。今回は追加の線形方程式を解かず、通常warm dual simplexの非退化時だけ、有限幅変数の境界違反を1+upperで割って行を順位付けする。無限幅は従来の絶対違反。RESTORATIONと退化時の既存Bland順は保持し、違反の適格条件FEAS・比率検定・最終certificate・deadline/pivot上限を変更しない。正規化scoreが小さくても最初の適格行を必ず保持する。

これは単純な有限幅の順位ヒューリスティックであり、Devexやsteepest-edgeではない。MIP_DUAL_RELATIVE_ROW既定off、CLIは0/1のみ受理。実発動counterを保存する。small warm行追加/費用変更のHiGHS oracleとFraction検査、選択MIP10/60秒で主解・下界・状態・仕事量を比較し、改善しなければ主評価へ広げない。別branchの成分仕事量配分は未採用なので混ぜない。全体優位・新記録は未達。

試作FPa5b27baa47e54171を全source/ELF/SHA固定。全248Rustはoff/on両成功・4ignored。選択10秒5モデルはmad0.3766→0.2512/下界ほぼ0同値、pk117/0、markshare1/0、Nidda13590.827113447798/−1216923.270104406、SP97884197292.16/594690451.7794694は同値。10主解/10証明は元MPS独立厳密/公式補助全有効。
行追加100モデル/1000再最適化=1100状態、費用変更100モデル/1000再最適化=1100状態はHiGHS・Fraction照合で失敗0。行追加1000状態でnormal dualの新counter発動を確認。成功状態を独立モデル2200件とは数えない。helper2個はruntime build-infoでlibrary fpを照合して実行前にsource/ELF/SHA固定。60秒と全188対/監査を継続中、まだ未採用。

同じhelper/入力生成seedのflag0側も検証し、行追加は従来warm2640pivot/0.040160937秒→新2878pivot/0.041198769秒、費用変更は37035pivot同値/約0.0976→0.0975秒。両側とも2200状態照合成功だが、新しい普遍的な速度改善は示さない。
60秒madは両0.108・pk1両11・Nidda両13590.8271、markshare両optimal約1。markshareの公式MIPLIB objective1を検証用途だけで確認（https://miplib.zib.de/instance_details_markshare_4_0.html、2026-11-16取得）、optimizerへ答えは渡さない。GMUの主解−2577545.7294→−2474573.8276の悪化は60秒でも持続。binkarの10秒改善は60秒では6865.969997→6867.120016へ小悪化した。全188と残る改善例・悪化例の対照を完了して判定する。

最終判定: 棄却。全188件10秒は主解7改善・7悪化・174同値、下界0改善・8悪化・122同値・58比較不能。両側の状態は171 time_limit /11 resource_limit /1 optimal /5 numerical_errorで一致。376証明・119出力点は元MPS独立検査で全有効。60秒追加対照はbppc65→64、glass4 2400021800→2200017900、lotsize14636546→14331368と改善する一方、GMUの持続悪化とbinkarの逆転を認め、汎用改善と判断しない。追加10点/10証明も全有効。コードは実験branch/凍結sourceだけに残し、mainには採用しない。

60秒native比較（mad,pk1、CPU42/1thread/16GiB）: mad Rust0.108/ほぼ0、HiGHS0.0984/0、SCIP0.1048/ほぼ0、GLPK0.2168/ほぼ0。pk1 Rust11/0、SCIP11/6.0918093、GLPK14/3.4011105。HiGHSの報告14.999999/1.9408279は公式相対検査を通るが、GMP絶対行違反1.000000008e-6が共通閾値1e-6を僅かに超えるため主評価の有効解に数えない。8点中厳密7有効・1無効、公式8有効、Rust2証明有効。監査起動のtaskset指定誤記を修正し再実行した。許容誤差は変更しない。
