# 長時間評価ランナー (protocol v1)

## 固定条件

公式MIPLIB 2017 Benchmark Set v2、240問。入力SHA256はdata/manifest.json。比較baselineはHiGHS 1.16.0 / SCIP 10.0.2 / GLPK 5.0。各3600秒、1スレッド、CPU affinity固定、8枠 (CPU 8..15)、1ケースの全プロセス合計16 GiB cgroup memory.max、swap禁止、プロセス単位RLIMIT_ASも16 GiB。CPU quotaも1コア。検査・ラッパーも同じケースのcgroupに入る。

各問題のCPUは元manifestのindex mod 8で決め、ソルバー間で同じCPUを使う。コアごとのキューは順序固定。グローバルflockで同時に2つの評価coordinatorが走らないようにし、最大8ケースを保証する。空いた別コアへの移動や難しい問題の省略はしない。

内部時間制限は最適化の3600秒。入力・import・出力用に外部watchdogの猶予60秒を加える。検査は別ステージで最大600秒。solver内部時間、外部実時間、CPU時間、最大RSS、cgroup集計ピークを全て保存し、検査時間は解法時間に含めない。未完了のシフト幾何平均は3600秒で評価し、OOM/クラッシュ/未対応/誤答を除外しない。

HiGHS/SCIPのgap設定は相対・絶対とも0、seed 0、スレッド1。HiGHS small_matrix_value=1e-12。GLPKは公式testscript v1.0.4 run_glpk.shに合わせcutsとfeasibility pumpを有効化する。GLPKの公開MIP seed設定は使わず標準の決定的初期化を使用。全パラメータと版/外部コード版をケースごとに保存。

v0の5秒smokeはRLIMIT_ASのみ、GLPK cuts/FP無効だった。v1は群全体のメモリを制限し、GLPKの設定も公式testscriptへ合わせたため、v0とv1の結果を同一系列として混ぜない。最終baseline全体をv1で新規測定する。

GLPK比較driverは汎用APIを使う独立Cプログラム (GPL-3.0-or-later)。探索は変更せず、境界を最大1回/秒観測してログの丸め誤差を避ける。最後の観測境界と経過時間を報告し、取得できない境界はnull。生成ノード数を処理ノード数と混同しない。Rust本体にリンク・呼出しをしない。ソース・ライセンスはtools/glpk-benchmark。

## 検査とstatus

primaryは元MPSと保存JSONの十進数をGMP有理数で読み、全行を絶対誤差で検査する。HiGHS reader/math.fsumの診断も残すが、primary判定には使わない。変数境界・制約・整数性の絶対許容誤差1e-6、目的値1e-8*(1+abs(obj))。native optimalかつprimary合格かつ数値gap<=1e-7だけをverified_optimalとする。statusとnative primal/dualを残し、不合格のnative primalを検証済みprimalに混ぜない。gapは共通式abs(primal-dual)/max(1,abs(primal))。

公式MIPLIB checkerも独立したMPS reader/GMP有理数で検査する。linear=1e-6/int=1e-6を明示。公式checkerのlinear/boundはpos/neg活動とsideでスケールする相対検査なので、primaryの絶対検査と区別して両方保存する。exit codeだけで合格とせずCheck SOLの全項目を確認。形式未対応や検査失敗/timeoutは解の誤りと区別する。最適性証明の厳密有理検証はこれらのprimal checkerでは行わない。

監査で、GLPKの目的定数RHS符号がCPLEX/HiGHSと逆であることを合成例で確認し、比較driverで定数だけ正規化した。公式checkerにも入力形式上の制約がある。全240問には非ゼロ目的定数RHSとOBJSENSEがないことを確認した (artifacts/mps-extension-audit.json)。

## 準備と起動

```
cc -O2 -Wall -Wextra -Werror tools/glpk-benchmark/main.c -lglpk -lm -o target/glpk-benchmark
make -C third_party/miplib-checker CXX=g++ LINKCXX=g++ USROFLAGS=-O2
cargo build --release
# ソースをcommitしてからsnapshot作成
.venv/bin/python scripts/benchmark.py create artifacts/runs/baseline-v1-20261115
.venv/bin/python artifacts/runs/baseline-v1-20261115/snapshot/scripts/benchmark.py run artifacts/runs/baseline-v1-20261115
```

createは入力ハッシュを検査し、実行バイナリ・全評価スクリプト・Rustソース・第三者ソースをsnapshotへコピーする。git SHA、snapshot SHA、共有ライブラリSHA、依存package版、機械、全コマンドを保存。実行中のsnapshotを編集しない。共有.venvやGLPK等のライブラリを更新しない。新規依存が必要なら別環境を作る。再開時はsnapshot/共有ライブラリのhashを再検査する。

長時間baseline中の重い開発・コンパイル・探索実験はCPU 8..15を避け、例として`taskset -c 32-63 ...`を使う。短期smokeはCPU 0で実施する。

status: `.venv/bin/python scripts/benchmark.py status artifacts/runs/baseline-v1-20261115`

coordinatorが停止した場合も、同じsnapshotのrunを再実行する。coordinator/case/stageのPIDとLinux start_ticks、PGID、boot_idで同一性を確認し、動いている同じ最適化を待つ。観測timeoutや古いファイルだけを理由に再起動しない。終了済みsummaryは失敗も含め再実行しない。stage終了状態・stdout/stderr・資源を原子的に保存。cgroup memory.eventsでOOMを識別する。

## 保存と期限

完全な出力はartifacts/runs内に保存する (大容量のためgitignore)。experiments以下にconfig・進捗・集計・結果/解のSHA indexをcommitし、PRコメントに結果とAI資源量を報告する。最終的に圧縮archiveとhashを作成し、専用リポジトリ内で取得可能にする。

比較3ソルバー×240問×1時間を8枠で回す最悪平均90時間 (コア固定による偏りと検査時間は別)。Rust全240問も最悪平均30時間。保存精度更新後の再測定を考慮し、最終Rust版は遅くとも2026-11-19 UTC中に固定し、再測定・検査・報告の余裕を確保する。期限は2026-11-22 14:59 UTC。

出典: https://miplib.zib.de/download.html (testscript v1.0.4)、https://www.scipopt.org/doc-10.0.0/html/FAQ.php (相対実行可能性許容誤差)、https://www.ibm.com/docs/en/cofz/12.9.0?topic=extensions-objective-sense-name-offset-in-mps-files (負RHSの目的定数規約)。


2026-11-15 保存精度プロトコル更新: GLPKの15桁raw解を採用せず直接API sidecarを採用し、全比較ソルバーの2^53以上の整数は全桁JSON整数で保存する。旧baseline-v1は保存精度の理由でsupersededとして保持し、最終比較には新規baseline-v2を用いる。詳細はdocs/SERIALIZATION_PLAN.md。資源条件とライブラリ版は同一。
