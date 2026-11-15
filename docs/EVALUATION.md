# 評価仕様 v0（実装前の事前登録）
公式 https://miplib.zib.de/download.html のBenchmark Set v2全240問。圧縮・展開MPSのSHA256、リスト、URLを保存。既知値は評価専用。名前のSHA256順で先頭160問を開発、残り80問を検証。検証は改善選別に常用しない。最終評価は240問全件。

主評価: 各実行1物理コア、壁時計3600秒、アドレス空間16 GiB、seed 0、相対gap停止1e-6。同じCPU集合・資源制限。外部監視は猶予5秒後強制終了。同時実行最大16、コア重複なし。開発用1/10/60秒は主評価と混ぜない。

解決数、独立検査済み実行可能解数、primal/dual bound、gap、時間、ノード、RSS、終了コードを保存。timeout/OOM/crash/unsupported/invalid_solution/numerical_failureを全件報告。
gap=|p-d|/max(1,|p|,|d|)。未解決を3600秒とした時間のシフト幾何平均（shift=10秒）、PAR-2、共通解決部分の時間を併記。失敗を高速解決に数えない。
独立検査器は元MPSの境界・制約・整数性・目的値を検査し許容差・最大違反を保存。解検査のみを最適性証明と呼ばない。証明状態は小問全列挙・外部比較、将来の証明ログで補強。

SCIP・HiGHS・GLPKの導入可能な安定版を次の環境固定コミットで確定。版、ソースcommit、ビルドフラグ、設定全文、バイナリハッシュ保存。原則デフォルト、1スレッド、同じ時間・メモリ・gap。変更時は理由と再実行を記録。

環境: aarch64 Neoverse-V2 64コア、247 GiB RAM。
根拠: https://miplib.zib.de/set_benchmark.html 、 https://miplib.zib.de/CHANGELOG.html 、 https://webdoc.sub.gwdg.de/ebook/serien/ah/ZIB/ZR-12-05.pdf
性能優越は測定前に主張しない。

## v1: 2026-11-15 環境・比較実装の固定
- 実インストール版: SCIP 10.0.2（PySCIPOpt 6.3.0）、HiGHS 1.16.0、GLPK 5.0。完全設定と共有ライブラリSHA256は `environment.json`、`scip-options.set`、`highs-options.txt`。
- 比較APIラッパーは `scripts/solver_worker.py`。GLPKは `scripts/glpk_driver.c` により公開APIの最良ノードboundを倍精度で記録する。コールバックは観測専用。GLPKドライバは比較専用、Rust探索とは接続しない。
- 初期GLPSOL CLI試行（logs/baseline-smoke-*）はdual boundの丸めと欠測があるため主比較に用いない。v1以降は全4ソルバーを再測定する。最適終了時はprimal=dual、非最適時にbound未取得ならnullとして報告し推測で埋めない。
- `benchmark.py` は各実行に同じCPU affinity、16 GiB RLIMIT_AS、ソルバー時間制限と外部壁時計制限（+5秒）を設定。timeout時はプロセスグループ全体を終了。生ログ、最大RSS、解、元MPSの独立検査結果、終了状態を保存。読み込み・起動時間は外部壁時計に含み、各ソルバー内部時間も保存する。
- HiGHSはthreads=1、parallel=off、random_seed=0。SCIPはparallel/maxnthreads=1、lp/threads=1、randomseedshift=0。GLPKは単一スレッド、presolve=on、他はglp_init_iocp既定値。全比較で相対gap停止1e-6。Rust初版はgap停止未実装で完全探索または制限終了。
- 初期開発比較は開発160問のうち圧縮ファイルサイズが小さい順に3問（markshare_4_0, enlight_hard, pk1）。この選択は性能結果の観測前に行った。小規模結果を全240問への優越性の根拠にしない。
- 独立検査はHiGHSの元MPS読込のみを使用し、最適化を呼ばずPythonで境界・整数性・行・目的値を再計算。変数名で対応付け、絶対許容差1e-7、目的値のみ相対許容差1e-7。検査器が成功しても最適性証明とは扱わない。
- 公式Downloadsページはbenchmark.zipをBenchmark Set v2（Feb 2023公開、Jun 2019更新問題）と明記。公式リストとzip全件一致を取得スクリプトで検査。`benchmark-manifest.json`に全ハッシュ、`benchmark-v2.test`に公式リストを保存。
