# 標準形RHSの丸め誤差による早期数値停止

2026-11-17、統合修復候補cb0ba03a36c3f07eとは独立した診断のみのブランチ。現mainへの変更ではない。

開発rd-rplusc-21は既存設定でroot numerical_error、LP1回/0pivot。固定列除去off、カット一括off、整数前処理offでも同じだった。入力検査をtraceに追加すると、621列/125899行のProblemでRHSにNaN、列/費用/上下限の検査項目には異常がない。

shifted_boundの拒否理由を最大16件表示すると、c44/c45/c48等でbinary-shift-summationの誤差1.05〜1.54e-9が内部基準1e-9を超えるためNaNになっていた。診断ログと元結果を保持。正しさの基準や問題自体は変更していない。

次は、NaNの代わりに外向きの保守的変換を返せるか、元問題の境界証明との対応も含めて調べる。単に基準を上げたり、RHSを無根拠に変更して実行可能性を作らない。独立Fraction/全列挙/GMP点/親claim誤差なし検査を通すまで採用しない。現在の診断は原因表示のみで、新しい解や性能改善の主張はない。

片側行だけの既定無効prototypeを作り、Source指紋一致/有限の係数誤差box/消失した十進項なしの場合に外向きRHSを構成する。等式/範囲行は対象外。最初のtestはsparse checkoutのfixtures不足で停止、tests/examples追加後の全333 Rustテストは成功4ignored。まだ実MPSや独立Fraction行証拠は未検証、未採用。

この調査で現main121に、係数0.99999999999999999999がf64整数1へ丸まりexact_integralを保つため、連続x∈[0,2], a*x<=1, min -xで報告dual=-1が厳密最適-100000000000000000000/99999999999999999999を上回る例を確認した。整数xと連続yを結ぶ1.00000000000000000001*x-y=0,y<=1の例はnumerical/dualNoneへ既存ガードが撤回する。正しさ優先で別branch decimal-integer-coefficient-safety（main121基準）の修正へ進み、このRHS prototypeは検証前のまま保持する。

小数係数安全性の最終候補の188比較を待つ間に、`scripts/verify_rhs_interval.py` を準備した。元MPSをFractionで読み、dumpのbinary64 RHSが `sign*b - sign*hatA*shift + max_box(sign*(hatA-A)*x)` 以上であることを許容なしで検査する。等式/範囲、行名/index不一致、重複shift、非有限値、非保守的なRHSは拒否する。現時点は構文確認だけで、Rust prototypeからのdumpとの照合・改ざんテストは未実施。prototypeをmainへ採用したり正しさを主張したりしない。通常shared targetは小数係数安全性fb986443b5/8b579cacb175412dであり、LP prototypeのbuildで上書きしていない。


## 2026-11-18: main122統合後の候補
main122安全性修正をd233193567で取り込んだ13c70f62b37f5c5fは337テスト成功。rd-rplusc-21は旧0pivot/numeric→10秒10573pivot/time、主解・下界なし。最初1024件の分数検査/10改ざん拒否が成功。全dump対応867583f71ca3ccf1も337成功、全55193適用が元MPSの許容なし分数検査に合格。全dumpのI/O費用を含む8552pivotを性能差とは扱わない。
固定大値+小数係数を持つ200生成MILPのflag0/1全400出力は不正下界/最適/実行不能誤判定0、最適7→103、GMP131→159、全436適用RHSの厳密検査有効。整数全列挙と有理数連続端点で解析し、Decimal精度100で正確なMPSを書いた。
既定有効候補3dc80cbe0e/9b884c5d9667c934を凍結して同じ検証を再実行中。main未採用。従来1e−9基準は変更せず、片側行の保守的な別表現だけを追加。元係数下界証明/未証明時の親下界維持と主解継続を保持。次は337テスト・全行/生成MIP再検証と、dumpなしflag0/1開発188対・全点GMP/公式で判断する。


## 最終判断
9b884c5d9667c934を採用する。337テスト、55,193全適用の分数証拠/10改ざん拒否、200生成MILP400出力/290GMPと436行証拠、1200列挙/1028実親証明/8224改ざん/GMP1200、188対376出力/128点が全有効。主解全同値、numeric5→4、下界3改善1悪化は適用0のため直接効果としない。専用CPU44/dumpなしrd10秒は0→10949pivot、主解・下界なし。詳細はRHS_INTERVAL_RESULTS.md。mainへ統合して再build/再検査/未起動snapshot更新に進む。
