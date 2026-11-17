# 通常dual再最適化のDevex行選択

親PR100/coreFP0722aa10b34775e6。以前の有限幅行正規化は棄却済み。今回は通常dual_runだけで、違反/重み平方根の順位を使う。違反の適格条件FEASと退化時の従来Bland順、比率検定、反転、refactor、certificate、deadline/iteration上限は変えない。RESTORATIONのdualには適用しない。MIP_DUAL_DEVEX既定off、CLI0/1のみ。

各dual呼出しの初期基底変数を参照集合、全行重み1にする。既存BTRANによる価格走査から参照集合の非基底列の係数二乗和を集めfloor1を適用。pivot pに対し新重み=max(1,測定重み/d[p]^2)、他行はmax(既存重み,新重み*d[i]^2)。推定/測定比が9超、100m（最低25）pivot超または非有限計算なら、現在基底で参照集合と重み1を再初期化する。追加FTRAN/BTRANはしない。固定非基底列が参照集合に残る場合だけ、その列の係数も測定する。重みが大きくても最初の適格行を保持し、score<FEASでも行を落とさない。

数学的更新とframeworkの一次実装資料: ERGO-Code/HiGHS commit f3f113d230429a8c0014b12ab0dd1a3847e58c55、highs/simplex/HEkkDual.cpp:1520-1548,2137-2192,2273-2301、HEkk.cpp:2260-2290、HEkkDualRow.cpp:614-632。MITライセンスと該当箇所をartifacts/dual-devex-literatureに保存。https://github.com/ERGO-Code/HiGHS/tree/f3f113d230429a8c0014b12ab0dd1a3847e58c55 。Rust実装は新規に書き、C++コードをビルド/呼出し/委譲しない。これはdual steepest-edgeの完全な行ノルム更新ではなく参照集合Devex。

補助一次資料: Qi Huangfu, High performance simplex solver (2013), University of Edinburgh, https://www.maths.ed.ac.uk/sites/default/files/atoms/files/huangfu.pdf ; Huangfu/Hall ERGO 2016 slides https://www.maths.ed.ac.uk/hall/ERGO_301116/ERGO_301116.pdf p40 ; Parallelizing the dual revised simplex method, 2018, DOI10.1007/s12532-017-0130-5 (CC BY4.0)。文章/図は転載せず方法を検討。

全Rust有効/無効、100行追加モデル1100状態と100費用変更モデル1100状態をHiGHS/Fractionへ照合し、実発動counterを保存。選択MIP10/60秒で主解/下界/状態/仕事量と時間を比較し、改善しなければ採用しない。既存ソルバーとの全体優位・新記録は未達。

試作FP02850befc50883c0を全source/ELF/SHA固定、補助2ELFもruntime library FP照合・source/SHA固定。行追加100モデル/1000変更=1100状態と費用変更100モデル/1000変更=1100状態をoff/on各々HiGHS/Fraction照合、失敗0。行追加warm pivot2640→2445、時間0.0399809→0.0398886秒（小差で速度優位は未立証）。実発動1000calls/2445updates/142reset。費用変更は37035pivot同値、dual Devex発動0で負の対照。2200独立モデルではなく200ベースモデルと変更状態。

選択10秒5モデル10点10証明は元MPS独立GMP絶対/公式補助すべて有効。pk1両17/0、mad両0.3766/ほぼ0、markshare両1/0、SP97両860211691.52/588126071.599。Niddaは13590.8271134→13856.8995027へ主解悪化、下界-1216923.270同値、両numerical_error。この差は60秒でも持続し、未解決の回帰として記録。markshare60秒両optimal約1で同値。pk1/mad60秒を完了して判断する。

60秒pk1主解11/0同値、nodes366844→437210（約19%増）、総pivot5580657→4930812。mad主解0.108/ほぼ0同値だがnodes124344→116628へ減少。markshare両最適1、38.56→38.93秒で改善なし。Nidda主解悪化は残る。効果が混在するため名前別のgateを作らず、全188対を同一凍結版で測って採否を判断する。

v2の費用修正: v1は退化時に既存Bland順を使う場合にも全行のsqrt重み順位を計算していた。Bland時にはscoreを参照しないためsqrtだけ省き、非退化時の順位・全weight更新・reference・比率検定・証明は維持する。v1の全188測定は凍結版で続行し、この費用修正版を別FPで記録する。


## 最終の開発比較と棄却判断（2026-11-17）

PR101と平方根の費用修正を合わせたFPc923fb2d5360335eでも開発188例を10秒・1スレッド・16GiBで比較した。121点と376証明が元MPSの独立検査をすべて通過した。主解は9改善・7悪化・172同値、双対境界は7改善・3悪化・120同値・58比較不能。終了状態は両版ともtime_limit171、resource_limit11、optimal1、numerical_error5。短時間では改善と悪化が混在し、60秒のreblockは-34820763.9524から-32826998.8171へ悪化して短時間の改善が逆転した。Binkar、glass4、Niddaにも持続する悪化がある。neos-911970、n5-3、h80、lotsize等には改善があるが、解けた数の増加も全体優位も確認できず、今回はmainへ採用しない。名前による切替条件は作らない。試作ソース、出典・ライセンス、失敗・悪化も含む測定結果を保存する。

## 正の同率比率で大きいpivotを選ぶ試作

親PR101、試作FP1a770c6d2feb097b。通常dualの比率が厳密に同じで1e-10以上の場合だけ、大きい分母を優先する。ゼロ比率・既存の退化順序・restoration・比率の適格条件・許容誤差は維持した。Harris比率検定ではない。MIP_DUAL_POSITIVE_TIEは既定off。全Rust検査255件成功・4件ignoredを有効/無効で確認した。

行追加100モデル1100状態を有効/無効でHiGHS/Fraction照合し失敗0、warm pivot2640→2639、約0.0401秒で実質的改善なし。1000呼出しで同率比較106回。費用変更100モデル1100状態も両方失敗0、37035pivot同値で発動0。独立モデル数と変更後の状態数は区別する。

選択10秒5例と60秒3例で元MPSの主解・双対境界に改善なし。既存の数値エラーも解消しない。この規模で効果を確認できないため全188例へ拡大せず、mainへ採用しない。Devex併用版の60秒3例を合わせ、22点と22証明すべて元MPS独立GMP絶対検査・公式補助検査で有効。

次はpivot順変更を重ねる前に、数値エラーの原因と既存カットの有効性・適用範囲を調べる。最終240例の自作版は未開始。SCIP/HiGHS/GLPK全体超え・新記録は未達である。
