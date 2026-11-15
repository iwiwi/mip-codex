# 評価仕様 v0 (2026-11-15)

機械: Linux aarch64、Neoverse-V2、64コア、約247 GiB RAM。共有ハードウェアの詳細は各実験に保存。

公式データ: https://miplib.zib.de/download.html の benchmark.zip (version 2) と benchmark-v2.test、240問。取得ファイルのSHA256を保存。名前や既知目的値をアルゴリズムに使用しない。

最終評価の予定: 全240問、各ソルバー1スレッド、1問3600秒、16 GiB、同じCPU affinity、seed 0 (対応する場合)。独立CPUで最大8ジョブまで並列実行し、実際の同時実行数を記録する。変更時はここに理由を書き比較を再実行する。短期開発は10秒または60秒を別の実験系列にする。全評価は時間見積もりから逆算して早期開始する。

開発/検証分割: ファイル名SHA256の先頭8桁を整数化しmod 5 = 0を検証、残りを開発に固定。検証群を改善判断に常用しない。別途合成小問題を正しさに使用。

比較対象: SCIP、HiGHS、GLPK。バージョンと全設定は導入後に固定して記録する (未導入)。デフォルト挙動に時間/メモリ/スレッド/seedを明示設定。最適性許容gapは0、実行可能性・整数性許容誤差は版と共に保存。

保存: solver/git SHA/version、入力SHA256、コマンド、実時間、CPU時間、最大RSS、終了コード、status、primal/dual bound、gap、nodes、解ファイル、独立検査結果。生ログを残す。unsupportedをinfeasibleと扱わない。

指標: 検証済最適解数、実行可能解数、失敗内訳、時間の10秒シフト幾何平均 (未完了は上限時間)、最終primal/dual/gap、相対primal/dual品質。誤答があれば目立つ形で報告し速度優位を主張しない。外部の既知値は評価器だけで扱う。

根拠: 公式セットは240問。原論文では4時間上限も分析しているが、期限内の反復と全比較を両立するため1時間を初期選択。
出典: https://miplib.zib.de/set_benchmark.html ; https://doi.org/10.1007/s12532-020-00194-3

## 導入版と短期測定 (2026-11-15)
- HiGHS 1.16.0 (highspy、git f3f113d)、SCIP 10.0.2 (PySCIPOpt 6.3.0)、GLPK/glpsol 5.0。Python側の固定依存はscripts/requirements-comparison.txt。
- 比較APIはscripts/comparison_worker.pyに限定。Rustの依存はserde/serde_json/flate2のみ。
- 短期smokeは開発群のpk1、mad、markshare_4_0を5秒で測定。選定は小規模の入出力・制限・検証経路の確認用。性能優位の評価用ではない。
- 各プロセスを同一CPU affinityとRLIMIT_AS=16 GiBで逐次実行。OMP/BLASスレッドを1に制限。内部時間制限に加え、読込・終了・ラッパー処理に60秒を加えた外部watchdogを置く。超過時はプロセスグループを強制終了し終了コードを残す。
- Rustは初期密表エンジンのため、表の推定256 MiBを超える問題をresource_limitとする。この実装上の制限を隠さず集計する。最終評価では疎LPへの移行が必須。
- 最適性gap: HiGHSのmip_rel_gap/mip_abs_gap、SCIPのlimits/gap/limits/absgap、GLPKのmipgapを0に設定。Rustは整数性1e-7、返却解の制約検査1e-6、最終数値gap検査1e-7。これらは浮動小数点による判定であり厳密有理証明ではない。
- 独立Python検査器は元MPSをHiGHS readerで再読込し、制約活動をmath.fsumで計算。絶対制約・変数境界・整数性許容誤差1e-6、目的値誤差1e-8*(1+abs(obj))。最適化は呼ばない。HiGHS readerのsmall_matrix_valueを1e-12に設定。既定1e-9ではmomentum1の7係数が省かれるため。全入力のRust readerとの照合を記録。
- CLIのoptimal/infeasibleは数値的探索結果を表す。LP非有界の場合はMILP非有界を断定せずunbounded_relaxationで停止。時間・ノード・反復・資源・数値異常を別statusにする。
- 参考仕様: https://ergo-code.github.io/HiGHS/dev/options/definitions/ ; https://www.scipopt.org/doc/html/PARAMETERS.php

## 疎LP採用の比較 (2026-11-15)

実装commit ff6c959を固定し、ソース領域cleanで計測。`MIP_BENCH_SOLVERS=rust-dense,rust-revised,highs,scip,glpk`で同一バイナリのLP方式を切替。各5秒/1コア/16GiB、逐次実行。既定ノード・LP反復上限を撤廃し時間制限まで探索する (明示上限は指定可能)。

最終smoke: artifacts/smoke-20261115-025135。全15runの最適性証明0、返却14解は独立検査で全て合格。

| 開発問題 | 密表Rust primal | 疎Rust primal | HiGHS | SCIP | GLPK |
|---|---:|---:|---:|---:|---:|
| pk1 | 46 | 29 | 約15 | 19 | 18 |
| mad | 解なし | 0.4938 | 0.208 | 0.386 | 0.1712 |
| markshare_4_0 | 約20 | 4 | 約5 | 約5 | 7 |

疎版の探索ノードは順に12275/11567/333944、密表版470/5/8688。LP経路が異なるためノード数だけで速度倍率を断定しない。

大きめの開発3問: artifacts/smoke-20261115-025252。密表の30n20b8は表メモリ推定でresource_limit、50v-10/binkar10_1は根LP中に時間切れ。疎版は3問全てで有限双対境界 (約1.56640765/2879.06568685/6637.18802694) を得て探索し、5秒で74/713/70ノード。6runともprimalなし。30n20b8の疎版最大RSSは約57.4 MiB。

追加の根LP照合: 上記6問の根LPはHiGHS relaxationと一致 (artifacts/revised-miplib-root-check.jsonl)。合成根LP200問では目的値が3エンジンで一致し、LP計算時間のみの幾何平均比は密表/疎=21.24。合成・小規模開発測定を全MIPLIBの成績に外挿しない。

採用: 疎エンジンを既定にする。理由は全検証で誤答/数値異常なし、密表より規模と解品質を改善したため。比較ソルバー全体への優位性は未達成。ノード間基底再利用、前処理、カット等は今後の課題。

## 長時間baseline v1開始

2026-11-15 UTC、全240問×3比較ソルバーの720ケースを開始。固定条件とv0からの変更理由・再開方法はdocs/EVALUATION_RUNNER.md。snapshot source commit c431aa9、ソースclean。config・依存・機械・保存時点の進捗はexperiments/baseline-v1-20261115。

最終評価はcgroup合計16GiBを加え、GLPK cuts/FPを公式testscriptに合わせ有効化したv1で統一する。以前の5秒smokeを同じ系列に混ぜない。全体成績・誤答/未検証の監査は完了後に行う。

## ノード間基底再利用 (2026-11-15)

source17c5872を固定し、開発CPU0/5秒/1thread/RLIMIT_AS16GiBでcold/warmと比較solverを逐次測定。GLPKは現行cuts/FP有効。この短期系列は長時間v1のcgroup全体測定とは区別する。

| 問題 | cold Rust primal | warm Rust primal | HiGHS | SCIP | GLPK |
|---|---:|---:|---:|---:|---:|
| pk1 | 29 | 約20 | 約15 | 19 | 21 |
| mad | 0.4938 | 0.2576 | 0.208 | 0.4208 | 0.3512 |
| markshare_4_0 | 4 | 約2 | 約5 | 約5 | 8 |

最終の小規模15run: artifacts/smoke-20261115-035415。全runが時間制限、証明0、返却15解は絶対誤差Python/公式GMP双方に合格。pk1ではGLPKのdual約0.241がRustの0より強く、primalの比較だけで全面優位と扱わない。raw gapはsolver固有定義も含むので、共通gapをartifacts/warm-final-summary.jsonに別途計算。

大きめ3問: artifacts/smoke-20261115-035532。
- 30n20b8: cold71/warm51ノード、両方primalなし、dual約1.56640765で同じ。改善は確認できない。
- 50v-10: cold680ノード/primalなし、warm15591ノード/primal8244.969954。dualは両方約2879.065687。
- binkar10_1: cold69ノード/primalなし、warm1845ノード/primal6944.740016。dualは両方約6637.188027。

新しい2解もPython/GMP双方に合格。全21runで証明0、返却17解全て検証済み。単なるノード数比を速度倍率と同一視しない。warm/coldはLP最適頂点とbranch経路が異なる。

初回版source554a057の失敗 (madのdual停滞、markshareのprimal悪化) もartifacts/smoke-20261115-034804に保持。dual退化判定をbetaへ修正し、Bland選択と構造依存pivot予算のfallbackを加えた後に再測定して採用した。35基本テスト、3000 bound/RHS変更のcold照合、1000小問題/200中規模のHiGHS比較、200合成根LP/6開発MIPLIB根LP照合が成功。全体優位は未達成。

## 境界伝播と有理数による絶対誤差再監査 (2026-11-15)

source fd31fce、各5秒CPU0/1thread/RLIMIT_AS16GiB、warm有効を共通として伝播だけon/off。artifacts/smoke-20261115-043915 と044009。全12runが時間制限、証明0。以下のprimalは最小化、返却10解は元MPSへの新GMP絶対検査・公式相対検査とも合格。

| 問題 | 伝播なしprimal | 伝播ありprimal | 伝播なしdual | 伝播ありdual |
|---|---:|---:|---:|---:|
| pk1 | 約20 | 約20 | 0 | 0 |
| mad | 0.2576 | 0.3520 | 約0 | 約0 |
| markshare_4_0 | 約2 | 約2 | 0 | 0 |
| 30n20b8 | なし | なし | 1.56640765 | 43.33557298 |
| 50v-10 | 8244.969954 | 8975.549969 | 2879.065687 | 2879.065687 |
| binkar10_1 | 6944.740016 | 6871.430005 | 6637.188027 | 6637.188027 |

伝播で50v-10の86ノード、binkarの7ノードをLPなしで棄却したが、処理量も増え、結果は混在。既定は無効のまま、MIP_PROPAGATION=1での実験用機能として保持。LP呼出し数lp_solvesと処理ノードnodesを分離した。比較solver全体への優位性は未達成。

検査精度の訂正: 従来のPython浮動小数点絶対検査は桁落ちで誤合格しうる。新primaryは元十進数をGMP有理数で検査する絶対版で、不能なら未検証とする。公式相対検査は補助として残す。基底再利用PRの17解と今回10解を再監査し、26/27が新primary合格、27/27が公式相対検査合格。**過去のHiGHS pk1解1件は新primary不合格**: 行違反は正確に125000001/125000000000000 = 1.000000008e-6、許容1e-6を超える。旧浮動小数点値は9.999999974752427e-7だった。許容誤差を緩めず、前節のHiGHS約15は新基準では検証済primalから除外する。Rust返却解は全て新検査に合格。再監査結果はartifacts/propagation-primal-audit-summary.json。

長時間baselineの保存解19件も新primaryで19/19合格 (04:39 UTC時点)。稼働snapshotは変更しておらず、この監査は別ディレクトリへ保存。未完了ケースは引き続き同基準で監査する。

## 丸め・修復による主解探索 (2026-11-15)

source6a6f961のablationはartifacts/smoke-20261115-044736。各5秒CPU0/1thread/RLIMIT_AS16GiB、warm有効・伝播無効。主解修復on/offだけを変更した。全12run時間制限、証明0。返却10解は新GMP絶対検査と公式相対検査に合格。

- 50v-10: primal8244.969954→3777.599989、dual2879.065687は同じ。初回主解は0.5957→0.00666秒。
- pk1: 最終primal約20は同じ。初回主解0.00842→0.000408秒。
- mad: 最終primal0.2576は同じ。初回主解0.3907→0.00108秒。
- markshare_4_0: 最終primal約2、初回主解約0.0004秒でほぼ同じ。
- 30n20b8: 両者主解なし、dual1.56640765、49ノードで改善なし。
- binkar10_1: 最終primal6944.740016、dual6637.188027は同じ。初回主解0.2649/0.2690秒で改善なし。

丸め+座標修復だけの先行版 (source2c4142f、smoke-20261115-044502/044543) でも50v-10の同じ主解改善を得たが、pk1では修復候補を得なかった。整数固定LPによる連続修復を追加して初回主解を改善。補助LPはRust実装を使い、主解候補だけを採用し、dualやinfeasibleを木の証明へ使わない。root最大50ms・以降256ノード毎最大1ms、256補助pivot制限。初回主解時間はsolver内部開始からの時間で、解品質は最終primalと併記する。

採用: 主解修復を既定有効へ変更。source e6a83c2を固定した最終比較はartifacts/smoke-20261115-044926、6問×4solver各5秒、CPU0/1thread/RLIMIT_AS16GiB。全24run時間制限、証明0。比較ソルバーのバージョン/設定は保存済みで、GLPK cuts/FP有効。長時間cgroup系列とは区別する。

| 問題 | Rust primal | HiGHS primal | SCIP primal | GLPK primal |
|---|---:|---:|---:|---:|
| pk1 | 約20 | 不合格 (native約15) | 19 | 21 |
| mad | 0.2576 | 0.2080 | 0.4208 | 0.3512 |
| markshare_4_0 | 約2 | 約5 | 約5 | 8 |
| 30n20b8 | なし | なし | なし | なし |
| 50v-10 | 3777.599989 | 3629.309987 | 6799.659986 | 4741.989989 |
| binkar10_1 | 6944.740016 | 6751.210022 | 6749.290015 | 6846.460044 |

下界の弱さは残る。50v-10のdualはRust2879.065687に対しHiGHS3224.774951、SCIP3217.921879、GLPK3083.301177。binkarもRust6637.188027に対し他3solverは6703以上。30n20b8はRust1.56640765、GLPK111、他2solverは有限dualなし。最終primalだけで総合優位と扱わない。全体優位は未達成。

返却20解の再監査は新primary19/20、公式相対20/20。HiGHS pk1は前回と同じ厳密行違反1.000000008e-6で不合格、検証済primalから除外。Rust返却5解は双方合格。全raw値と共通gap (分母max(1,abs(primal)))、不合格/主解なしをartifacts/repair-final-summary.jsonへ保存。49 Rustテストと最終1000小問題+200中規模照合は失敗0。
