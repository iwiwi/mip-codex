# 厳密積和の次の高速化案

PR25の正確なf64積和と元十進数precision guardを維持し、68limbへの移行頻度を減らす。現行は単一f64に厳密に載らなくなった時点でWideへ移り、通常の十進係数のrow shiftでもコストが増える。

次はPoint/Pair/Wideの3段階を試す。Pairは和そのものを2個のf64で**厳密に**保持する。TwoSumの残差をもう一度TwoSumで加え、第3成分が非ゼロになる場合だけ既存68limbへ移る。Pairを収束させたhi+loの一度の丸めとTwoSum残差から最近接・上下界・絶対丸め残差を得る。overflow/非正規化/極端な指数は既存Wideへ戻す。

非厳密な積は、最下位の真の積bitがsubnormal範囲より下にならないと証明できる場合だけ、FMAで厳密な積の2成分を作れるか検討する。それ以外は既存の整数limb積に戻す。まず和だけのPairで測定し、FMAは独立した検証の後で判断する。

1200個の独立Fraction bit列fixture、相殺・tie・overflow・underflow・元十進数反例を維持。性能のために元MPS精度、未証明infeasibleの扱い、最近接丸めを変えない。最終版の比較は独立target/source/binary provenanceを保存し、固定18問とmarkshare60sで判定する。


## 初回測定とFMA版

2成分の和だけを加えたb4b79095は、同時18問A/Bのprimal全同等、markshare main35.640s / Pair36.131s。Pointの通常経路まで3回TwoSumを行う余計な仕事があり、改善として採用しない。

Pointのtail=0、または追加の残差=0を短縮する。FMAは元のbinary値を m*2^e とした時 ea+eb>=-1074、かつ丸めた積が有限の場合だけ使用する。この十分条件では真の積とその丸め残差の全bitがsubnormal下限以上で、FMA残差は厳密に表せる。その他は既存68limbに戻す。途中のpair追加が第3成分を要求する場合も、変更前のpairと元の積をWideへ渡すため情報を落とさない。

参照した数学は Ogita, Rump, Oishi (2005), Accurate Sum and Dot Product, SIAM J. Sci. Comput.26(6),1955–1988, Algorithm3.5 (TwoProductFMA)、DOI10.1137/030601818。著者公開一次資料 https://ogilab.w.waseda.jp/ogita/math/doc/2005_OgRuOi.pdf 。ソルバ/ライブラリのコードはコピーせず、数式をRustで独自実装する。一般のdouble-double近似に置き換えず、3成分が必要な場合を検出して厳密limbへ戻す。

既存1200 fixtureに加え、FMA許可条件の直前/直後、subnormal×大数、積overflow後の相殺、3成分の救出、偶数丸めtie、整数×小数の計2400独立Fraction fixtureを追加する。合計3600ケースで最近接・最小上下界・丸め残差をbit比較する。
