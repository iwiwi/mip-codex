# Fresh factor ratio失敗の残差診断

2026-11-15。PR53後のglass4/app1-1について、再分解直後のprimal_ratio失敗時のbasis/全column/rhs/point/方向を保存する診断を追加した。計算判定は変更していない。MIP_LP_RATIO_DUMPで未存在の一意ファイルに保存、MIP_LP_TRACEで境界違反を記録する。unscaled raw失敗の標準LP dumpも追加。

glass4の2基底では通常加算が残差0を返す行でも、Fractionによるbinary64係数と点の厳密積和は残差inf norm12.5625/4.6875だった。基底変数の下限違反はそれぞれ1.35064/0.02176（scaled単位）。app1-1は行残差1.42847e-16と小さいが、基底下限違反1.16218e-8がFEAS1e-8を超えていた。加算精度と真の基底実行可能性を分けて調べる。

再現した疎LUでr=b-Axを既存のexact::dot（積も丸めず最後だけ最近接）で計算し、B*d=r、x_B+=dを最大4回試す診断を実行する。これはまだ本探索へ統合しない。境界/行許容値は緩めない。初期値と各補正後の厳密積和残差、基底境界違反、元のratioが通るかを保存する。

方法の参考：Carson/Higham, A New Analysis of Iterative Refinement and Its Application to Accurate Solution of Ill-Conditioned Sparse Linear Systems, 2017, DOI 10.1137/17M1122918、著者所属大学の公開稿 https://eprints.maths.manchester.ac.uk/2563/1/paper.pdf （2026-11-15閲覧）。一般的な残差補正の数式を参照し、既存コードをコピーせず独自Rustと汎用faer LUで再現する。収束や全MILP精度改善を仮定しない。

再現補正はglass4の両基底で2回目に元のratioを通し、基底境界違反を1e-12以下へ減らした。app1-1は4回でもratioを通らず、単純な残差補正では解消しない。glass4の一例は残差inf norm自体が4.6875→5.5398へ増えたため、inf normだけの改善とは主張しない。ゼロRHSの小さい成分を含む通常の成分相対誤差も1になる。実探索版を試すなら、r/(1+|b|+|A||x|)の最大値が悪化しないこと、基底の境界と元ratioが既存FEASのまま通ることを候補受理の条件とし、最終元単位/元問題検査を維持する。
