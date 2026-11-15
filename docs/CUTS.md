# 証明付き二値coverと入力の合計

## カットの成立条件

原MPSの係数・RHS・RANGESが厳密な整数で、各値と合計が絶対値2^52以下の行を対象にする。元の十進文字列を検査し、f64へ丸めただけの整数は対象にしない。変数は原境界と整数性から0/1の範囲に限られるものだけ。連続変数や多値整数を含む行は今回は除外する。

原行を向き付きの上限制約 `sum(c_j*x_j)<=b` とする。負係数を `z_j=1-x_j` で補数に変換し、正係数では `z_j=x_j` とする。packingの容量は `b-sum(c_j<0のc_j)`。正の重みの合計が容量を厳密に超える集合Cには `sum(z_j for j in C)<=|C|-1` が成立する。

選択はLP値に基づく二つの貪欲順で行い、成立はi128の整数演算で確認する。生成側から独立したverify関数は、Cを全選択したときの元行の最小活動量を直接再計算し、それが元RHSを超えることを検査する。最小活動量は、選択した正係数の値と、選択しなかった負係数の値の合計。導出行の係数±1とRHS、元行、向き、集合を照合する。

数学的定義の参考: [Kati Wolter, Cutting Plane Separators in SCIP](https://scipopt.org/download/slides/SCIP-cuttingPlanes.pdf) の0-1 knapsack節。外部ソルバーの実装は移植せず、選択・検査・LP再最適化を自作Rustで実装する。

## 実行と独立検査

MIP_COVER_CUTS=0/1、既定無効。根で最大4round×32cut、時間は全体5%または0.25秒の小さい方。row最大16384項、cover最大256変数、選択work上限2,000,000。適用可能性を調べる走査はcut_secondsに含めず、全体secondsに含む。

追加行はLP用Modelだけに保持する。主解は常に元Modelと元MPSで検査する。LPの返却状態だけで新たなinfeasible/optimalを宣言せず、証明済み行を追加した後に主探索がLPを解く。構造の変わったwarm基底は既存の互換性検査で棄却してcoldへ戻す。全LP数・pivot等に前処理LPを含める。

各cutは元行index/name、upper/lowerの向き、cover indexと導出行を出力する。verify_cover_cuts.pyは元MPSの十進文字列をPython Fractionで独立に読み、原境界・整数性と強制集合の最小活動量を検査する。主解がない終了も対象。将来のbenchmark snapshotとsmokeは、不合格または未完了なら下界を除外して最適/実行不能の主張を採用しない。偽のinfeasible結果に不正cutを添えた実プロセスでguardを検査した。稼働baseline snapshotは変更しない。

## 入力重複項の修正

MPSの重複項を逐次f64加算した旧版では、`1e16+1-1e16` が0になった。元の `x<=1, x<=2, min -x` を誤ってoptimal -2/x=2とし、GMP絶対検査で行違反1を確認。目的係数だけの重複でも旧optimal0に対して真の最適値-2があり、`-0.1-0.2+0.3` は真に0なのに旧unbounded_relaxationを返した。

小整数項は既存の固定長整数累算器で合計し、最後に最近接・偶数丸めする。一般の十進重複項は元文字列を保持し、汎用num-bigint 0.4.8で整数mantissaと10の冪へそろえて合計してから、一度だけf64へ変換する。単独項で多倍長演算はしない。重複十進数は最大10000桁/指数絶対値10000で、それ以上は明示エラー。非ゼロが0へunderflowする入力または合計、有限範囲を超える合計もエラーにする。LPの最適化や有理数LP証明を外部へ委譲したものではなく、LPコアは引き続き浮動小数点である。

[num-bigint](https://docs.rs/num-bigint/0.4.8/num_bigint/struct.BigInt.html) と依存num-integerはMIT OR Apache-2.0。用途は入力の汎用整数演算のみ。Cargo.lockで版を固定し、license/sourceはartifacts/cover-dependency-licenses.jsonへ保存。

3反例の元MPSを一変数のFraction検査で独立に照合し、修正版の最適値-1/-2/0と点を確認した。行を持つ反例は元MPSのGMP絶対検査も合格。RHS省略の目的のみMPSは通常checkerの構文制限で検査不能だったため、そのログを残し、GMP合格件数へ混ぜない。原MPSハッシュ・before/after・検査はartifacts/cover-counterexample-audit.json。

## 検証と採用判断

96 Rustテスト、f64合計のPython Fraction624fixture、元十進合計612fixture、整数重複12順序を照合。カット生成250行の全256二値点も確認し、不正証明/連続変数/原データ不明を棄却。Python検査4テストとstage回帰2テストが成功。

最終source670f15bで全240 MIPLIBファイルの行列・目的・境界・整数性が独立HiGHS readerと一致した。この全件検査は入出力の正しさだけで、holdoutの性能調整には使わない。

純二値の同じ1000問 (repair/diving/probeを無効) を比較:

|設定|探索node合計|LP総数|solve時間合計秒|cut本数|cutが出た問題|
|---|---:|---:|---:|---:|---:|
|off|5320|5320|0.106726|0|0|
|on|3676|5424|0.201837|2969|477|

全1000問はHiGHS/整数全列挙と一致し、全2969cutは元MPS Fraction検査に合格。nodeは減るがLP総数が増え、極小問題の時間は悪化する。混合1000+中規模200も失敗0。

開発18問のon/offは全36run時間制限・証明0、返却26解はGMP絶対/公式相対とも合格。ただし全18問で実際のcutは0本。pk1主解19→17の差をcutの効果と扱わない。整数行だけの基本coverでは現実問題の信号が足りず、既定化しない。混合行、非整数係数の証明付き変換、lifting等は未実装。

probeを有効にした5探索×18問も測定し、全90run時間制限、返却62解は双方合格。gmu2問では全方式がprobeの主解を保持したが、その後の改善はなかった。mad等で悪化もあり、既定depth/downを維持。全結果はartifacts/cover-summary.json。

最終既定 (cut無効) の8問×Rust各5秒は全時間制限・証明0、返却7解は両検査合格。主解/下界は前PR13と同等。比較3solverは前回同資源smoke-20261115-071815を再利用し、今回は再測定していない。全体優位は未達成。
