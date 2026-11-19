# 撤収記録 — 2026-11-19 UTC

研究とベンチマークを終了した。撤収時のプロセス表と測定用cgroupには稼働中の研究ジョブがなく、作業中の子エージェントもいなかった。終了票がない監査は未完了として保存した。EC2・EBSは利用者による回収のため残してある。

## 推奨コミットと保全した枝

再開時の基準は **main の `a326be92a59eea70d31d057a7121a5ea9fef6746`**（PR #134、Rust fingerprint `b180426ba9745d9f`）。公式評価用実行ファイルのSHA256は `cf0232f5ebc28dfaf80a06f9d28625b129b80511fd852315bbbb7d1556abf06c`。この撤収コミットは文書のみ。

| 用途 | 枝とコミット | 状態 |
|---|---|---|
| 等式・有限範囲のRHS区間対 | `ranged-rhs-interval@96011348f0edf9f83774d862d75e731b0e4266bb` | 既定有効候補、188問の独立監査が未完了、未採用 |
| 元LP点の精度停止 | `exact-lp-point-diagnostic@ca77d0e2803f99da494313d64ec1b5976abd11b2` | 診断のみ。自由連続変数の補正は実装前 |
| 一般整数ポンプ回収 | `general-pump-limited-repair@063f9f815db5cacd9bda205102de83b8cf825689` | 各8問×3方式、品質改善なし、不採用 |
| グラフ頂点探索 | `graph-vertex-portfolio@1175a8e6ee2d947f49554d1ca0b963c46c6d9ddf` | 既定無効。300秒でz26が悪化したため未採用 |
| グラフ近傍の独立診断 | `cover-choice-heap@b2d00ef6f45648c9b833f1d72b5c0a140b1e4805` | 自作z26解−1198と検証資料を保全。公表値との同値 |

旧MIR作業ツリーの未コミット疎LU差分は `retirement/mir-matrix-wip-20261119@f3bda96c401d75b2a605b547d85410bf3d5eeb21` に保全した。撤収時に再検証していない。

残るローカル仮説・却下枝は回収用の `unpublished-history.bundle` と `GIT-REFS.json` で復元する。GitHubへは確認したソース差分とこの文書を通常のpushで保存した。

## 評価の状態と既知の問題

- 正式native v2は240問×3ソルバーの720実行が完了。設定SHA256 `39551a3b530d8c4e1ed4eb4069c361285e929a84a74d73966bb4c574b43c125c`、結果 `6acf941e6bc3c151cb183c3e5f849af4ea823515bbe987c41e383e7cf45c1bab`。結果の `verified_optimal` はHiGHS 145、SCIP 119、GLPK 26。SCIP・GLPKは各1点が一次絶対検査で不合格。HiGHSにメモリ制限1件、その他のエラーはHiGHS 2、SCIP 1、GLPK 37。個別の終了状況・原因は各caseの原ログを参照。
- native管理プロセスのENOSPC障害から復旧し、時刻だけで事前指定したGLPK 1件も別設定で完了した。再測定は点のGMP・公式検査に合格。元の720結果は保持する。比較集計時の置換規則は [BASELINE_DISK_RECOVERY_20261118.md](BASELINE_DISK_RECOVERY_20261118.md)。
- 自作正式240問は **0/240、未起動**。凍結設定 `artifacts/runs/farkas-row-projection-final-20261118/config.json` のSHA256は `e484a2e3d1c00140a6044740d75715c8c08e80f9ba7a593857c085d87af66ead`。古い正式候補は履歴用。開発188問と個別品質を調整に使わなかった検証52問の分割は `data/manifest.json`。
- main #134の開発188問対照は全376証明と136返却点のGMP・公式検査が完了。crnaの早期数値停止は改善したが300秒では主解なし。HiGHS・SCIPの300秒有効解に及ばなかった。詳細と時間依存の損失は [FARKAS_ROW_PROJECTION_DEFAULT_RESULTS.md](FARKAS_ROW_PROJECTION_DEFAULT_RESULTS.md)。
- RHS区間対の既定有効候補はRust368通過・4無視、厳密生成400出力の検査完了、有効側は33最適・101不能・66数値停止。開発の最適化376/376は終了したが独立監査は93/376（93証明、30点がGMP・公式合格）で終了票なし。事前走査版と遅延版の完了した比較・損失は同枝の `docs/PAIRED_RHS_PRESCAN_RESULTS.md` と `docs/PAIRED_RHS_LAZY_RESULTS.md`。生成した混合小数の5例は既存保護による数値停止。公式相対checkerは一部生成MPS構文に非対応。既知の数値限界と旧版で発見した誤答は [ORIGINAL_WITNESS_FIVE_LIMITATION_20261118.md](ORIGINAL_WITNESS_FIVE_LIMITATION_20261118.md)、[INTEGER_BOUND_SAFETY_HYPOTHESIS.md](INTEGER_BOUND_SAFETY_HYPOTHESIS.md) と個々の研究資料を参照。leo1の診断では元等式に約3.245e−5の実違反が残った。
- 正式外CDC 1時間は旧固定Rustで頂点探索OFF −291/−429、ON −291/−424（主値/双対、両点GMPと証明有効）。HiGHS −280/−1111（内部計時3628.97秒）、SCIP −237/−11744、GLPKは3660秒watchdogで終了。元ログは回収資料の `graph-vertex-cdc-*`。z26の自作−1198は既存公表値との同値。新記録および正式セットでの総合優位は未達成。
- 全103正式・smoke設定の内訳と未起動・未完了、旧版の比較や誤答の生データは回収ディレクトリの `EVALUATION-STATUS.json`、`MANIFEST.json`、GitHub各枝の `docs/*RESULTS.md` と `artifacts/*evaluation/manifest.json` に索引を残した。検査済み点とソルバーの最適性宣言と探索全体の形式証明を区別する。

## 実行環境と取得先

Ubuntu 24.04.5 LTS、Linux 7.0.0-1014-aws、aarch64、Neoverse-V2 64論理CPU、約247 GiB RAM。Rust/Cargo 1.100.0、Python 3.12.3、GNU C++ 13.2。長時間測定はcgroup v2、CPU 8–15、各1スレッド/16 GiB/3600秒、swap 0、pids 512、wall猶予60秒、検査600秒。短期開発では別の固定コアと明示した制限を使用した。

| 対象 | 取得先・固定情報 |
|---|---|
| MIPLIB 2017 Benchmark Set v2 | [MIPLIB downloads](https://miplib.zib.de/download.html)。`benchmark.zip` SHA256 `c756eefd544d83b31809306b45d3549a1a5b9378e6aa78b68738b1a3b6a418fa`、`benchmark-v2.test` SHA256 `0ee9758dba64bb1689cc121c794ee547992a046bcabe1cce9ea79158375dabd5`。240入力のSHA256は `data/manifest.json`。探索用入力は回収アーカイブ内の `data/raw/exploratory` |
| HiGHS / SCIP | PyPIの `highspy==1.16.0` と `PySCIPOpt==6.3.0`（同梱SCIP 10.0.2）。NumPy 2.5.4、SciPy 1.18.1。`scripts/requirements-comparison.txt`。当時のネイティブライブラリ実体のSHA256は正式configの `library_sha256` 全147件 |
| GLPK | [GNU GLPK 5.0 source](https://ftp.gnu.org/gnu/glpk/glpk-5.0.tar.gz)、SHA256 `4a1013eebb50f728fc601bdd833b0b2870333c3b3e5a816eeba921d95bec6f15`。Ubuntuの `libglpk-dev:arm64=5.0-1build2` を比較driverにリンク。driver SHA256 `329ec481ab751fa1b9cb540b5a986fb41b2b0a299d2bb707960cfdbd66ceffd4` |
| GMP checker | [MIPLIB testscript 1.0.4](https://miplib.zib.de/download.html)、ZIP SHA256 `37b21a7f1ee968aed60675fb070c9286eccc6a953c8bc50120c9246b052b47b9`。ソースと絶対誤差版の改変は `third_party/miplib-checker`。公式実行ファイルSHA256 `df5c3728704d6858560eb36f2395c8220b8df66d4960b1a209656815c9c099e0`、絶対版 `fc29d7831b1cc7894f5c80cd412fee655cb54ef972f125e867aaaaba720bd6b6` |

探索用の4入力はMIPLIBの個別インスタンス配布物。圧縮MPSのSHA256は次の通り。

| 入力 | SHA256 |
|---|---|
| [cdc7-4-3-2](https://miplib.zib.de/instance_details_cdc7-4-3-2.html) | `7cb233a7b3df00065ebe43aff1e991ff2884e5defa8cc951b0c568ab25a754f4` |
| [sorrell7](https://miplib.zib.de/instance_details_sorrell7.html) | `11b427bdaf61e4d7b54268b26bdf37707a446df82e6e7cbcce88751ad1997b45` |
| [sorrell8](https://miplib.zib.de/instance_details_sorrell8.html) | `f4c2a95d93d72f166c67fd5736006e62681e27661fd07425801e212c28b1eeb5` |
| [z26](https://miplib.zib.de/instance_details_z26.html) | `63ca4683d1311d5e1390472d5dfb880af2a6100dd11a8a08006d90fa1e27b783` |

比較専用ソルバーには共通時間とスレッド数を渡す。具体的なseed、gap、cuts等の指定は `scripts/comparison_worker.py`、`tools/glpk-benchmark/main.c` と凍結設定・各jobのoptionsを参照。Rustの最適化本体は外部ソルバーを呼ばない。

## 再開する場合の手順

以降は将来の再開用コマンド。今回の撤収では実行していない。対象ホストのパス、CPU、cgroup権限を確認し、古い実験タグを再利用しない。

```bash
git clone https://github.com/iwiwi/mip-codex.git
cd mip-codex
git switch --detach a326be92a59eea70d31d057a7121a5ea9fef6746
sudo apt-get install build-essential pkg-config python3-venv libglpk-dev libgmp-dev zlib1g-dev
rustup toolchain install 1.100.0
python3 -m venv .venv
.venv/bin/python -m pip install -r scripts/requirements-comparison.txt
python3 scripts/fetch_miplib.py
git diff --exit-code -- data
cargo +1.100.0 build --locked --release
cc -O2 -Wall -Wextra -Werror tools/glpk-benchmark/main.c -lglpk -lm -o target/glpk-benchmark
make -C third_party/miplib-checker CXX=g++ LINKCXX=g++ USROFLAGS=-O2
make -C third_party/miplib-checker MAINNAME=solchecker-absolute OBJDIR=obj-absolute \
  CXX=g++ LINKCXX=g++ USROFLAGS=-O2 USRCXXFLAGS=-DMIPLIB_ABSOLUTE
target/release/mip-codex build-info
target/release/mip-codex solve data/raw/benchmark/cod105.mps.gz 10
```

`target*`、Cargoキャッシュ、`.venv`、checkerのobj、公式入力は上記で再生成できる。バイナリの完全一致が必要なときは回収資料内の現行正式スナップショットを使い、configに記録された全SHAと照合する。パス、UID・GID、ライブラリSHA、CPU条件が変わるホストへは凍結設定をそのまま移して起動せず、新規タグで構成する。異なるビルド・測定資源の結果は比較時に明記する。

```bash
# 新しい計測を行う場合の例。createはsudo -nとcgroup v2を使用する。
.venv/bin/python scripts/benchmark.py create artifacts/runs/restored-rust-v2 --solvers rust
.venv/bin/python artifacts/runs/restored-rust-v2/snapshot/scripts/benchmark.py run artifacts/runs/restored-rust-v2
.venv/bin/python scripts/benchmark.py status artifacts/runs/restored-rust-v2
# 保存済み結果だけを集計する場合。
.venv/bin/python scripts/report_benchmark.py artifacts/runs/baseline-v2-20261115 \
  --replacement artifacts/runs/baseline-v2-disk-recovery-20261118 --output native-report.json
```

詳細は [EVALUATION_RUNNER.md](EVALUATION_RUNNER.md)、[BENCHMARK.md](BENCHMARK.md)、[FINAL_REPORT_SCORING.md](FINAL_REPORT_SCORING.md)。正式な比較報告を作る場合は自作240件の完了を確認し、未検証点・エラー・数値停止を含める。

## ホストから回収するもの

**`/home/ubuntu/mip-retirement-20261119/` をディレクトリごと回収する。** 具体的な収録物、SHA256と復元手順は同ディレクトリの `README.md`、`MANIFEST.json`、`SHA256SUMS`。ファイルごとに元の絶対パス、バイト数、SHA256、目的、再生成可否、Gitで復元できる場合のコミット・パスを記録した。GitHubの既存bundleから復元する資料はそのメンバー名も記録した。EC2/EBSを削除する前に別媒体へコピーし、ハッシュ照合する。
