# dual分母不一致後だけの新LU再試行

2026-11-16。main4556e45d6 / FP527633393c003557（PR76）から。dual_runでB^-Tの行から選んだ候補に対しB^-1列の分母が非正/非有限または残距離不正となった場合、eta更新が残り、時間/pivot余裕がある場合だけ同じbasisを元列からLU再分解し、双対候補選択を再開する。既に消費したflip/pivotは返さない。新LUでも失敗すれば従来どおりNumeric。MIP_DUAL_DENOMINATOR_REFACTOR=1限定、既定OFF。

根拠はniddaの停止traceでeta6更新後の分母−6.45と候補逆積の正符号が食い違っていたこと。残差補正・PhaseI再始動・区間伝播・完全ピボット回復は先の実験で追加の主解/dual改善がなく未採用だった。今回も節点増加だけを改善として数えない。

次: 同ELF0/1、10秒のnidda/app1/glass/crnaと純LP選択を比較。改善がなければ原出力を保存し棄却。改善した場合だけspent budget/deadline/親basis不変の回帰、元MPS点・証明検査、188開発/60秒へ進む。holdoutは使わない。全件nativeは凍結条件で継続、全240Rustの最新snapshotはparity-equality-final-20261116を作成/照合のみ。
