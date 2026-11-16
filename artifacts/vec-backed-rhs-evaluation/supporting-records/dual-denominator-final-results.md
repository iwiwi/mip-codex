# dual分母不一致後だけの新LU再試行

2026-11-16。main4556e45d6 / FP527633393c003557（PR76）から。dual_runでB^-Tの行から選んだ候補に対しB^-1列の分母が非正/非有限または残距離不正となった場合、eta更新が残り、時間/pivot余裕がある場合だけ同じbasisを元列からLU再分解し、双対候補選択を再開する。既に消費したflip/pivotは返さない。新LUでも失敗すれば従来どおりNumeric。MIP_DUAL_DENOMINATOR_REFACTOR=1限定、既定OFF。

根拠はniddaの停止traceでeta6更新後の分母−6.45と候補逆積の正符号が食い違っていたこと。残差補正・PhaseI再始動・区間伝播・完全ピボット回復は先の実験で追加の主解/dual改善がなく未採用だった。今回も節点増加だけを改善として数えない。

次: 同ELF0/1、10秒のnidda/app1/glass/crnaと純LP選択を比較。改善がなければ原出力を保存し棄却。改善した場合だけspent budget/deadline/親basis不変の回帰、元MPS点・証明検査、188開発/60秒へ進む。holdoutは使わない。全件nativeは凍結条件で継続、全240Rustの最新snapshotはparity-equality-final-20261116を作成/照合のみ。

V1 90afe65716942741を全ソース/ELF/build-info/SHAとともに凍結し、4件×0/1の同CPU41/10秒対照を完了。app1/glass/crnaは同品質。niddaは同じ70ノードで停止し、主解14054.945596894968→14251.287576392877へ悪化、dual−1216923.270104406は同じ。1.57→0.97秒の早い停止を速度改善と数えず棄却する。8出力の切断/格子証明と4整数点のGMP絶対/公式検査は全件有効。追加の188対照へは進めない。

原出力はprimary artifacts/dual-denominator-refactor-*。main4556e45d6/FP527は不変。次は現mainのprofileで計算時間を使う関数を確認し、探索経路を変えない費用削減を検討する。最終全240snapshotはparity-equality-final-20261116の作成のみ。全件nativeは10:43UTC243/720、継続中。
