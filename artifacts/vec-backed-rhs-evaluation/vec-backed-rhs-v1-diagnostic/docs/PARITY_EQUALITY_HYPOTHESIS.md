# 私有±2整数補助つきbinary等式の厳密偶奇解法

2026-11-16。問題名/既知解を使わず、全変数が整数、binary元境界が厳密0/1、全行が元データで厳密な整数等式、各行にその行だけへ現れる非binary整数変数がちょうど1個・係数±2、という構造だけを受理する。binary部分をGF(2)で消去し列数と階数が等しければbinary値は一意、各私有整数は元行からi128で一意に復元する。不整合/階数不足/不明精度/境界不一致/期限や仕事量上限では通常MILPへ戻り、実行不能は宣言しない。

復元値の整数性・元十進境界の内側・全行を検査し、有限目的値のみ返す。全整数解が一点以下でその点が実行可能なので最適。目的boundは元目的係数の外側区間を用い固定点のCertifierから計算する。証明にbinary/私有整数の添字、階数、pivotを保存。独立Python検査は元MPSのFraction係数からhighest-bit streaming消去を行う（Rustの列順RREFとは別実装）。正確な復元点と目的・boundも照合し、benchmark_validateへ組み込む。

V1 758e218bbc1ea6c4はenlight_hardの同CPU41/10秒で従来time_limit・点なし・bound16.4583から、optimal37/bound36.99999999999989へ、内部時間0.000115746秒。元MPSの独立階数100/100・一意性証明、GMP絶対と公式点検査が成功。これは新記録ではなく既知最適値の再現。全体優位性を示すものではない。

既定ON候補はLimits.parity_equalities、MIP_PARITY_EQUALITIES=0/1。最大2048行/2048binary、4096全変数、消去word更新2000万、全体残り時間の内最大0.05秒。通常binary/MIP構造へ条件を拡張しない。純LP検査は明示OFF。

次: 300小整数モデルで全列挙し、一意性の誤宣言を検査。元証明の改変拒否、min/max・目的offset・小数目的・負の私有整数・整数へ丸められただけの元係数/RHS/境界を検査。既存Rust/2700合成/500根LP/数値32、全188開発の同ELF0/1、同資源native比較後に採否判断。holdout入力は使わない。現main36ab93183/FP337の全240snapshotは作成のみ、全件nativeは継続。

既定ON候補FP527633393c003557を全ソース/ELF/build-info/SHAとともにparity-equality-final-diagnosticへ凍結。219 Rust回帰（300問題の全列挙を含む）、通常合成2700、根LP300設定500出力、数値32（18有効点/14精度未解決）は成功。別CLI生成200問題の全列挙とGMP点検査成功、90一意性証明と270改変拒否成功。小数目的・目的offset・min/max・負の私有整数を含む。

188開発の同ELF0/1はCPU3..7で進行、全出力の独立検査はまだ未完了。同CPU41/10秒/16GiB/1threadの4 solver実測smoke-20261116-102123-g_g338rpではRust optimal37、dual36.99999999999989、内部0.000142861秒/全process0.010745秒。SCIP optimal37、内部0.00358秒/全0.118669秒。HiGHS主解なし/dual36・10秒制限。GLPK主解37/dual25・10秒制限。最初のworktree smokeはGLPK driver既定パス不足で起動失敗し、原ログを保全して正しいMIP_GLPK_DRIVERを明示し4件とも再測定した。正式最終全240ではない。
