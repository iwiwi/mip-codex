# 変数間の積和相殺とLP変換の修正

整数伝播の主探索統合を保留して、旧mainの既存誤答を先に修正する。

- x=y=z=1に固定し、目的1e16*x+y-1e16*z。旧版はoptimal0、元MPS GMPでは目的1/目的誤差1。
- 同じ固定点で1e16*x+y-1e16*z=1。旧版は行のshiftを0と計算して誤infeasible。全固定点が原行を満たすため明確な反例。
- 固定x=1e16,y=1、free zでx+y+z=0。旧版の浮動点z=-1e16はGMPで行違反1。f64で必要な絶対精度の点を表せないので、修正版はnumerical_errorとし、最適/実行不能と断定しない。

既存の68limb厳密積和を使い、個々の積も中間丸めせず最後に一度だけ丸める。Modelの目的/offset、LPの変数shiftのRHS、主探索のdual+offset、revised LPのprimal/dual certificateの合計を対象とする。上下界は丸め残差の符号で最も狭いf64区間へ、dual合計は下へ丸める。LP RHS/幅を絶対誤差1e-9で作れなければNaNを数値検査へ渡しnumerical_errorにする。

通常の行チェックには保守的な前方誤差区間を使い、曖昧な場合だけ厳密積和へ戻る。積のoverflowが打消しで有限になる例、offset、min/max、bound-only/等式forcedを検査する。1200 f64 bit列の独立Python Fraction fixtureで最近接・狭い上下界・丸め残差・速い区間を照合する。

全Rust回帰、原MPSのGMP直接照合、合成小/二値/中規模、根LPのHiGHS比較、固定18開発問を行う。数値不具合の修正は性能悪化があっても元へ戻さず、正しさを保った範囲で高速化する。MPS reader/原データは変更しない。既存float探索全体の厳密証明化とは主張しない。


## 十進数との差を検出した試作の棄却

3f1cd1cb は未採用。元の十進数0.1とf64の0.1の差を見落としていた。
固定x=10^12,z=1の目的0.1x−10^11zで誤optimal 5.551115123125783e-6（正しくは0）、
等式0.1x=10^11で誤infeasibleを再現。`artifacts/exact-decimal-rejection` に試作出力を保存した。
修正版は十進数とIEEE rationalをBigIntで比較して係数・目的・境界の点区間を認識する。
整数格子に使う従来の小整数provenanceは別のまま維持する。
目的評価とrow shiftでは元係数および出力点の十進表現の保守的な誤差予算を課し、
保証できなければnumerical_error。RHS区間は外向きに緩和し、候補点は内側端点で検査する。
非点データがある主探索LPのinfeasibleは、元区間の零目的Lagrangian下界が正になる場合だけ採用する。
証明情報がないdense/メモリ制限/曖昧な証明はnumerical_errorとして未解決に残す。
大きい積overflowのfixtureはf64原始演算の検査には残すが、元十進モデルの最適性を主張しない。
元MPS readerを変更しないという当初案は撤回し、数学モデルを変えず精度provenanceを追加する。
