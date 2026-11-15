# 変数間の積和相殺とLP変換の修正

整数伝播の主探索統合を保留して、旧mainの既存誤答を先に修正する。

- x=y=z=1に固定し、目的1e16*x+y-1e16*z。旧版はoptimal0、元MPS GMPでは目的1/目的誤差1。
- 同じ固定点で1e16*x+y-1e16*z=1。旧版は行のshiftを0と計算して誤infeasible。全固定点が原行を満たすため明確な反例。
- 固定x=1e16,y=1、free zでx+y+z=0。旧版の浮動点z=-1e16はGMPで行違反1。f64で必要な絶対精度の点を表せないので、修正版はnumerical_errorとし、最適/実行不能と断定しない。

既存の68limb厳密積和を使い、個々の積も中間丸めせず最後に一度だけ丸める。Modelの目的/offset、LPの変数shiftのRHS、主探索のdual+offset、revised LPのprimal/dual certificateの合計を対象とする。上下界は丸め残差の符号で最も狭いf64区間へ、dual合計は下へ丸める。LP RHS/幅を絶対誤差1e-9で作れなければNaNを数値検査へ渡しnumerical_errorにする。

通常の行チェックには保守的な前方誤差区間を使い、曖昧な場合だけ厳密積和へ戻る。積のoverflowが打消しで有限になる例、offset、min/max、bound-only/等式forcedを検査する。1200 f64 bit列の独立Python Fraction fixtureで最近接・狭い上下界・丸め残差・速い区間を照合する。

全Rust回帰、原MPSのGMP直接照合、合成小/二値/中規模、根LPのHiGHS比較、固定18開発問を行う。数値不具合の修正は性能悪化があっても元へ戻さず、正しさを保った範囲で高速化する。MPS reader/原データは変更しない。既存float探索全体の厳密証明化とは主張しない。
