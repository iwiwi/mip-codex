# 出力された十進数の境界検査

main0178b0a1（RustコアはPR26と同じ、FP9445422755d296cf）の既存不具合を再現した。
元MPSのFX境界へf64::MAXの正確な309桁の整数を指定し費用0にすると、旧版はoptimal0を返すが、JSONの短い十進表現が内部値と異なりGMP絶対検査で境界違反8.145274237317043e290。
より小さい2^60+256（約1.15e18）でも短い出力と元整数に32の差が出る。beforeをartifacts/serialization-diagnosisに保存する。固定列省略の実験より、この既存の出力不具合を優先する。

## 実装

- 2^53以上の有限f64はすべて整数なので、元mantissa/exponentから正確な整数を作りJSON numberとして出力する。値・目的・dualで同じ方法を使い、丸めた下界を十進出力で上へずらさない。
- serde_jsonのarbitrary_precisionは汎用JSON I/Oのためだけに有効化。BigIntは既存の汎用数学依存で、LP/MIPの委譲はしない。
- point_errorはこの出力仕様に従い、整数の出力誤差を0として扱う。
- 変数境界を値の区間で速く検査し、曖昧な境界だけ実際のJSON数と、f64の境界±許容誤差の厳密和をBigIntで比較する。
- row/variableの比較は大きな境界へ許容誤差を足して丸めず、TwoSumで下から囲んだ差で判定する。元rowの境界精度がない場合はfail-closed。

## 検査

現時点でRust149テスト成功。正負MAX、費用1の目的/dual、2^60+256、2^100をDense/Revised・min/maxで検査する。
JSONの値がstringでなくnumberで、正しい全整数と一致することを確認。
Python Fractionで作った400の境界fixtureのうち337は、旧方式の丸めたbound+toleranceでは受理し得るが、元十進値では許容誤差を超える。新guardがこれらを拒否することを確認。
既存3600 exact-dot bit oracleも維持。次はCLI出力のGMP直接検査、合成2700/根LP300/固定18問/mark60、入力metadata比較の全240を確認する。
この修正は正しさのために採用し、検査負担や主解悪化があっても隠さない。根固定列の実験はその後に戻る。
