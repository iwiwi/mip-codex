# 一般整数の初回feasibility pump

2026-11-16。main36ab93183 / FP337114ffc2f9a6b7から開始。MIP_GENERAL_INTEGER_PUMP=1限定。既存binary pumpは維持し、非binaryの可動整数変数がある場合だけ別経路を使う。根かつ主解未発見、時間10%最大1秒、64投影/各4096pivot、可動整数1024/元LP非零200000以下。名前/既知解に依存しない。

一般整数xの丸め目標zとの距離d>=0にx−d<=z、−x−d<=−zを加えmin sum d。binary部分は従来の±x距離。元変数のsplit/shiftを使って補助行を構築する。直近8目標の繰返し時、分数性と固定seed順位でsqrt(n)最大32変数を許可範囲内の隣の整数へずらす。親warmは不変、局所の目的/不能/dualは本探索へ渡さない。元モデル候補検査を通った点だけ本探索の既存検査へ戻す。

V1 5ab573ffbea6d930は全ソース/ELF/SHA/build-infoをgeneral-integer-pump-v1-diagnosticへ凍結。enlight_hard、icir97_tension、crna、roll3000の各10秒対照で主解なし/下界不変。enlightは64投影0.04秒、icir/rollは初回局所LPのpivot上限で終了、crna10投影1秒。採用信号なし。

一般整数回帰3件（循環脱出・親LP不変、負の整数/連続混合、期限切れと整数不能で候補なし）は成功。負の整数の例は最初の丸めで点を得るため、実際の負整数距離投影の検査は別途必要。

次はMIP_GENERAL_PUMP_WARM_EXTENSION=1限定で、親の補助列番号を新構造列の後へ移し、新距離列と片側slackをbasisに入れる行/列同時拡張。作成後に通常minimize_warmのprimal再計算と全証明検査を通す。新目的なのでdual実行可能性を仮定しない。小数目標/自由変数の点とparent不変性を検証し、冷始動との同ELF性能を比較する。

開発適格性の分類には原MPSをHiGHS readerで読むのみ（最適化は呼ばない）。最初の分類スクリプトはpybind vector propertyを変数ごとにコピーし遅かったため停止し、境界配列を一度だけ読む版へ置換した。停止ログを保存。holdout入力は使わない。

V2 76329e24e1e72020の冷/温対照はicir/rollで最初の投影がwarm成功したが、いずれも主解なし。enlightでも改善なし。距離列拡張の両側目標・親不変・無効構造拒否と負の自由整数の実投影を追加し、lib62成功/3 ignored。性能採用せず保全する。次は単行専用±2整数補助を持つbinary等式をGF(2)で解く汎用候補/一意性検査。enlight_hardでは100変数/100独立偶奇式となり、診断の厳密Fraction検査で唯一の点の目的37を確認した。名前/既知解は実装に使わない。
