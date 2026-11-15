# 厳密積和の2成分経路とFMA

変更はsrc/exact.rsの厳密積和だけ。元MPSの精度ガード、丸めの意味、証明条件、探索戦略は変えない。
Pointで正確に載らなくなった和をPair(hi,lo)で**厳密に**保持し、第3成分が必要なら既存68limbへ戻す。
FMAは各入力の整数mantissaの指数ea+eb>=-1074かつ丸めた積が有限の場合だけ使う。
この十分条件では積の最下位bitまでf64のsubnormal範囲にあり、積の残差が失われない。
TwoSumの第3成分を捨てる通常のdouble-double近似には置き換えない。

数学の参照は[Ogita, Rump, Oishi (2005), Algorithm3.5](https://ogilab.w.waseda.jp/ogita/math/doc/2005_OgRuOi.pdf)。既存実装のコードはコピーしていない。指数の十分条件は真の積の最下位bitから独自に導いた保守的な条件。

## 正しさ

最終ソース50fe377d、source fingerprint9445422755d296cf。比較mainはee72cdd3、fingerprintcb68e6f8ab38ba3c。
専用worktreeと**別の実targetディレクトリ**を使い、CLI/library/sourceの一致を測定前に確認。

- Rust147テスト。独立Python Fraction fixtureは既存1200+追加2400の計3600ケースで、最近接・最小上下界・丸め残差をbit比較。
- 新fixtureはFMA許可境界の前後、subnormal×大数、積overflowの相殺、3成分の救出、偶数丸めtie、整数×小数を含む。
- 合成2700件（小1000/二値1000/中200/十進混合500）・純粋根LP300件（通常200/スケール100）のHiGHS/列挙/元MPS照合は失敗0。
- 元十進数の桁落ち回帰22件は従来どおり8有効点/14数値エラーの未解決/誤答0。
- 全保存68点はGMP絶対・公式とも全合格。286cuts/88runsの格子情報も独立検査合格。診断コマンドの失敗runに残った最初の点も検査に含むが、性能には数えない。

## 同時A/B：固定18開発問、各5秒

両版1 thread、16GiB、同じNeoverse V2マシンの専用CPU。全18問のprimalは同じ、改善0/悪化0。
解のない問題も同じ。gen-ip002は両版とも未証明infeasible節点を残したnumerical_error。
単位時間の節点数は増減があり、全体の速度改善とは主張しない。

8問のnative比較はPR25の同条件測定を明記して再利用する。両版で同じprimal/dualであり、SCIP/HiGHS/GLPKとの表は[PR25結果](EXACT_TRANSFORMATION_RESULTS.md)を参照。
HiGHS pk1の絶対誤差不合格もそのまま区別する。今回nativeを再測定したとは主張しない。

## 同じ探索仕事の処理時間

| 対象 | main | FMA Pair | 条件 |
|---|---:|---:|---|
| gen-ip002、追加3回の中央値 | 3.634016s | 3.127024s | 同じ37,993節点、約14.0%短縮 |
| gen-ip002、CPU交換 | 3.632341s | 3.142772s | 同じ37,993節点、約13.5%短縮 |
| markshare_4_0、60秒枠 | 35.737193s | 35.292987s | 同じ1,486,619節点、約1.2%短縮 |
| markshare_4_0、CPU交換 | 35.776949s | 35.247671s | 同じ1,486,619節点、約1.5%短縮 |

genの3回ではstatus/目的/dual/節点/LP数/pivot数/全変数値が両版で完全一致することを別途assertした。
mainの3回は3.664000/3.634016/3.632203s、Pairは3.127024/3.155459/3.123998s。
ただし最終statusはnumerical_errorのままで、解けた問題数や最適性証明の改善として数えない。
markshareは両版とも目的0.9999999999999206（元の整数目的1、許容誤差内）の最適性を証明し、1,494,216LPも同じ。
markshareの効果は小さく、一般的な大幅高速化とは主張しない。

## 棄却・測定失敗

和だけの初期Pair b4b79095は、main35.640sに対し36.131sで改善なし。
通常のPoint経路に余計なTwoSumを入れていたため短縮し、FMAを加えた最終版を採用する。
反復の最初のコマンドで同名問題を3回渡して出力先が衝突した。140503の両runは1点保存後にFileExistsError。
そのrunを速度比較から除外し、別プロセス・別ディレクトリの3回で再測定した。失敗のログは削除していない。

## 保存・再検査

全実験をartifacts/pair-accumulator-evaluationへ格納。relocation.jsonに移動前/後と全ファイルSHA256を保存し、移動時に一致を検査した。
元の生ログ/commandのパスは歴史記録として変更せず、再実行時はarchive prefixへ読み替える。
`pair-all-audit/summary.json`は56点/280cuts、`fma-extra-audit/summary.json`は12点/6cuts。
`fma-comparison-summary.json`に反復実測と一致assertの結果、各experiment.jsonにsource/binary/checker hashを保存。

次は保存済みchecked i128整数行伝播を主探索へ統合し、既定falseから検査・A/Bを進める。全Rust本測定の11月19日頃開始期限は維持する。
