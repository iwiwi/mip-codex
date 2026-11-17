# 標準形RHSの丸め誤差による早期数値停止

2026-11-17、統合修復候補cb0ba03a36c3f07eとは独立した診断のみのブランチ。現mainへの変更ではない。

開発rd-rplusc-21は既存設定でroot numerical_error、LP1回/0pivot。固定列除去off、カット一括off、整数前処理offでも同じだった。入力検査をtraceに追加すると、621列/125899行のProblemでRHSにNaN、列/費用/上下限の検査項目には異常がない。

shifted_boundの拒否理由を最大16件表示すると、c44/c45/c48等でbinary-shift-summationの誤差1.05〜1.54e-9が内部基準1e-9を超えるためNaNになっていた。診断ログと元結果を保持。正しさの基準や問題自体は変更していない。

次は、NaNの代わりに外向きの保守的変換を返せるか、元問題の境界証明との対応も含めて調べる。単に基準を上げたり、RHSを無根拠に変更して実行可能性を作らない。独立Fraction/全列挙/GMP点/親claim誤差なし検査を通すまで採用しない。現在の診断は原因表示のみで、新しい解や性能改善の主張はない。

片側行だけの既定無効prototypeを作り、Source指紋一致/有限の係数誤差box/消失した十進項なしの場合に外向きRHSを構成する。等式/範囲行は対象外。最初のtestはsparse checkoutのfixtures不足で停止、tests/examples追加後の全333 Rustテストは成功4ignored。まだ実MPSや独立Fraction行証拠は未検証、未採用。

この調査で現main121に、係数0.99999999999999999999がf64整数1へ丸まりexact_integralを保つため、連続x∈[0,2], a*x<=1, min -xで報告dual=-1が厳密最適-100000000000000000000/99999999999999999999を上回る例を確認した。整数xと連続yを結ぶ1.00000000000000000001*x-y=0,y<=1の例はnumerical/dualNoneへ既存ガードが撤回する。正しさ優先で別branch decimal-integer-coefficient-safety（main121基準）の修正へ進み、このRHS prototypeは検証前のまま保持する。
