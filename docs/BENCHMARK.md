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

## 擬似コスト分枝・子ノード順 (2026-11-15)

source6d138a2で4方式を比較。全方式でwarm/主解修復有効、伝播無効、1thread/RLIMIT_AS16GiB。6開発問はCPU0/各5秒 (smoke-20261115-045604)、追加12開発問はCPU1/各2秒 (smoke-20261115-045745)。追加12問は既存6問を除くdevelopmentの圧縮サイズ100000bytes以下から固定seed2026111503で測定前に抽出し、選択条件・入力hashをbranching-extra-selection.jsonへ保存。CPU/時間が違う2系列を時間倍率として混ぜない。

| 問題 | fractional/down | fractional/nearest | pseudocost/down | pseudocost/estimate |
|---|---:|---:|---:|---:|
| pk1 | 約20 | 約17 | 約20 | 約17 |
| mad | 0.2576 | 0.6628 | 0.2218 | 0.6356 |
| markshare_4_0 | 約2 | 約2 | 約2 | 約2 |
| 30n20b8 | なし | なし | なし | なし |
| 50v-10 | 3777.599989 | 3777.599989 | 3777.599989 | 3777.599989 |
| binkar10_1 | 6944.740016 | 7186.600012 | 6904.630026 | 7498.210051 |

下界は同じ。pseudocost/downの改善はmad・binkarで、nearest/estimateは改善と悪化が混在。追加12問ではpseudocost/downがgen-ip002を-4702.724985→-4703.110424へ改善し、他の主解の得られた5問は同等。n5-3/mc11/beasleyC3/pg5_34/bppc4-08の同方式primalは10425/13548/955/-13829.286851/117。nearestはbppc4-08を80へ改善する一方でgen-ip002を悪化、estimateはn5-3/gen-ip002/pg5_34を改善するが、coreのmad/binkarで悪化するため既定にはしない。

全72runはtime_limit64件、numerical_error8件、証明0。追加群のgmu-35-50とgmu-35-40は全4方式で根LPのnumerical_error。mcsched/csched008/timtab1/fhnw-binpack4-4は全方式主解なし。この失敗を集計から除かない。gmu-35-50はrevised rootで176pivot後に数値エラー、denseは30秒で時間切れ。独立HiGHS rootは両問ともoptimalで、目的値-2608070.315743009/-2406943.556342883。LPの数値処理を次に調べる。

返却44解 (core20+extra24) は新GMP絶対/公式相対とも全合格。52 Rustテスト、新3探索方式それぞれ1000小問題照合、estimateの中規模200、最終採用downの中規模200も成功。最終1000+200は採用後の既定ソースで再実行。raw/status/hash/検証はartifacts/branching-*と2つのsmokeに保存。

採用: 擬似コスト分枝を既定有効、floor側先行を維持 (source2f505c7)。主解が得られた11問では3改善・8同等、残り5問は主解なし、2問は数値エラー。学習値は順位だけに使い、下界や棄却には使わない。オプションMIP_BRANCHING=fractional、MIP_CHILD_ORDER=down|nearest|estimateで対照実験可能。

比較3solverは直前の同条件5秒測定smoke-20261115-044926 (本変更で参照solver/共有依存は変更なし) を参照。madのRust0.2218はHiGHS0.208より悪いがSCIP0.4208/GLPK0.3512より良い。binkarのRust6904.63は改善しても他3solver (約6749〜6846) に及ばない。今回その3solverを再測定したとは扱わず、全体優位は未達成。

## baselineの不合格を検出

05:00 UTC時点で22/720完了、うちcbs-cta/GLPKがinvalid_solution。nativeはobjective0/optimalと報告したが、元行c5862のupper0に対しactivity1.44935807712805。浮動小数点絶対、新GMP絶対、公式相対で不合格。保存native solution.rawをGLPK自身のread_mip/check_kktで読み直してもPB最大違反1.44935807712805 (row5862) を再現し、名前変換/JSON検査だけの不一致ではないことを確認。最適化は再実行せず、native statusと生ログを残して検証済primal/最適には数えない。baseline snapshotは変更せず、他ケースは継続。監査ソース・結果はartifacts/branching-baseline-invalid-audit。

## LP比率検定と微小目的係数の正しさ (2026-11-15)

source1e093b8。数値安定化と誤った最適宣言の修正で、許容誤差を緩めて成功扱いにしていない。詳細はdocs/NUMERICAL_STABILITY.md。

- gmu-35-50/35-40は旧全方式で根LP numerical_errorだった。Harris二段階比率検定と変数値の単位での誤差判定により、約0.02/0.013秒で根LP目的がHiGHSと一致した。
- min y-1e-12*x, x=z, x/z>=0, y∈[0,1]で旧両エンジンがoptimal0を誤宣言した。価格付けとray目的改善のスケールを修正し、両者unbounded_relaxationへ訂正。x=z=1e12,y=0の目的-1の点もGMP絶対検査で確認し、旧「最適」を実際に反証した。実行可能点0の検査だけでは見つからない誤りである。
- denseの行/目的正規化・元単位へのdual/Farkas復元も追加。中間試作のfalse infeasibleは回帰で検出して不採用とし、失敗ログを保存。最終60 Rustテスト、両エンジンそれぞれ1000小問題+200中規模照合、通常200/スケール変更100根LP、開発8根LPを通過した。

最終比較は8開発問×4solver各5秒CPU0/1thread/RLIMIT_AS16GiB、artifacts/smoke-20261115-052105。全32run時間制限、証明0。比較版/設定/生ログ/hashを保存 (GLPK cuts/FP有効)。

| 問題 | Rust primal | HiGHS primal | SCIP primal | GLPK primal |
|---|---:|---:|---:|---:|
| pk1 | 約20 | 不合格 (native約15) | 19 | 21 |
| mad | 0.3766 | 0.2080 | 0.4208 | 0.3512 |
| markshare_4_0 | 約2 | 約5 | 約5 | 8 |
| 30n20b8 | なし | なし | なし | なし |
| 50v-10 | 3777.599989 | 3629.309987 | 6799.659986 | 4741.989989 |
| binkar10_1 | 6920.290013 | 6751.210022 | 6749.290015 | 6846.460044 |
| gmu-35-50 | -112379.8182 | -2605437.3597 | -2603755.2784 | -2396821.7722 |
| gmu-35-40 | -878554.4542 | -2405426.3311 | -2402695.8198 | -2301097.5364 |

修正前pseudocost/downの5秒run (smoke-20261115-045604) と比べ、mad0.2218→0.3766、binkar6904.630026→6920.290013と悪化。他4問は最終品質が同等。LPの最適頂点/探索経路が変わり、単なるノード数比を速度倍率と扱わない。gmu2問は根LP失敗から主解/dualを返す状態へ改善したが、primalは他3solverより大幅に悪い。Rust dualはgmu-35-50 -2608070.315743、gmu-35-40 -2406943.556343。全体優位は未達成で、主解ヒューリスティックと下界強化が必要。

返却28解はGMP絶対27/28、公式相対28/28。HiGHS pk1の厳密行違反1.000000008e-6は今回も不合格、検証済primalから除外。Rust7返却解は双方合格。native値/検証済値/共通gapをstability-final-summary.jsonに分けて保存。安定化と既知の誤った最適宣言の修正を採用し、悪化した解品質も省略しない。

05:24 UTC時点で長時間baseline24/720完了、既知invalid_solution1、kernel coordinator生存。保存解21件を新GMP primaryで再監査し20合格/1不合格 (cbs-cta/GLPK)、検証不能0。最適化snapshotは変更せず監査を別保存 (artifacts/stability-baseline-audit)。

## 2026-11-15 列・行の単位と微小pivot

有限LPを誤って非有界とした3反例を修正。詳細は[LP_SCALING.md](LP_SCALING.md)。73 Rustテスト、両エンジン各1000小問+200中規模、通常200/scaled100根LP、開発8根LPが成功。追加の変数単位profile100問はrevised全成功し、返却点も元MPS GMP絶対検査に合格。denseは50成功/50 numerical_errorで未解決。全失敗・棄却試作を保存した。

source4bff42cの8開発問×Rust各5秒、CPU0/1thread/16GiB (smoke-20261115-061837) は全時間制限・証明0、返却7解はGMP絶対/公式相対とも合格。主解はpk1=20、mad=.3766、markshare≈2、30n20b8なし、50v-10=3777.599989、binkar10_1=6920.290013、gmu-35-50=-112379.8182、gmu-35-40=-933357.2996。直前Rustのgmu-35-40=-878554.4542以外は同じ。8問とも実際の列/行変換数0であり、この差をスケーリングの性能効果とは扱わない。

HiGHS/SCIP/GLPK比較値は同資源の直前smoke-20261115-052105を再利用し、今回再測定していない。gmu-35-40は各-2405426/-2402696/-2301098、gmu-35-50は-2605437/-2603755/-2396822でRustの主解に大幅な不足がある。全体優位は未達成。詳細値/前後差はartifacts/column-scaling-summary.json。

GMP絶対検査器だけ長い十進係数の32文字制限を除去し、近平行反例の正確な係数を検査可能にした。診断用Pythonの行境界取得をO(m²)から一度の取得へ修正 (許容誤差不変)。NumPy試作のJSON型エラーも保存し、型を保つ版の3回帰テストを通過。新検査器によるbaseline保存解監査は06:43 UTC時点33件中32合格、cbs-cta/GLPK不合格1、未検証0。稼働snapshotや共有ソルバーは変更しない。

## 2026-11-15 根LP潜り

固定開発18問で潜りon/offを比較 (8問×各5秒CPU0、以前抽出した追加10問×各2秒CPU1、1thread/16GiB)。再試行あり試作9e4d19aは全36run時間制限・最適証明0。終了主解はpk1 20→17、bppc4-08 86→72、残り16問同等 (主解なし5問を含む)。返却26解はGMP絶対/公式相対とも全合格。詳細と全試作はdocs/LP_DIVING.mdとartifacts/diving-summary.json。局所LP結果を本体の下界や枝刈りに利用しない。整数固定版29f548eも同じ2改善/16同等であり、返却26解は両検査合格。

bppc4-08を新規同CPU1/2秒で比較したHiGHS57、SCIP58、GLPK64は3解とも両検査合格し、Rust72はなお及ばない (smoke-20261115-065634)。ほかの3solver参照は前回同資源5秒smoke-20261115-052105を再利用し今回未再測定。pk1はSCIP19/GLPK21に対しRust17、HiGHSのnative約15は元MPS絶対検査不合格なので比較用primalから除外。gmu2問はRust主解を改善できず比較solverに大幅に劣る。全体優位は未達成。

潜りを既定で有効化する。78 Rustテスト、最終候補の1000小問題+200中規模は失敗0。従来の修復を潜りより先に戻して初解発見の遅延を減らした。局所探索の失敗や最大5%/0.25秒の消費は隠さず記録する。全件baselineのsnapshot/共有依存/CPU8..15は変更しない。

既定fb94450の最終8問smoke-20261115-070014は全時間制限、証明0、返却7解は両検査合格。gmu-35-40は-923666.5855で直前対照の-933357.2996より悪化した。追加の同CPU0反復で時間打切りの変動と潜り消費を検査する。他の主解は上記と同じ。最終既定1000+200と78 Rustテストは成功。

追加反復 (smoke-20261115-070155/070206/070216、CPU0/5秒、on/off各3回) は再試行有効時-923666.5855、無効時-933357.2996を全3回で再現。約0.15秒の再試行消費による悪化として、再試行を棄却しc58d292で整数固定版へ戻した。返却6解は両検査合格。採用版は最大256局所LPのみ、最終再測定を別保存する。

採用c58d292の最終smoke-20261115-070316は全8run時間制限・証明0、返却7解は両検査合格。gmu-35-40は-933357.2996へ戻り、CPU0/5秒の追加on/off各3反復 (070356/070407/070417) も全6run同じ-933357.2996。主解を悪化させた再試行を取り除いたことを確認した。78 Rustテストと最終1000+200比較は失敗0。全18問の対照・棄却試作・最終版は別々に保存し、最終値はartifacts/diving-adopted-summary.jsonを参照。

## 2026-11-15 探索順と主解専用probe

主探索はdepth/downを維持し、推定コスト順の最大4096ノード・全体20%/最大1秒の探索から、検査済み主解だけを受け取る。18開発問の同source対照は3改善/1悪化/14同等。pk1は17→19へ悪化、gmu50は-112379.8182→-2532435.2987、gmu40は-933357.2996→-1860472.4551、gen-ip002も改善。全36run時間制限・証明0、返却26解は両検査合格。短期比較の悪化を記録して採用し、全体優位とは主張しない。

最良下界/混合探索は未処理全ノードの下界を保持する任意機能として追加。probe無効で5方式×18問を比較し、pk1/50vの主解や下界が改善する一方、多くの主解悪化とgmuの主解喪失があったため、既定化しない。全90run時間制限・証明0、返却54解は両検査合格。子優先順だけの比較54runの返却37解も全合格。詳細と全結果はdocs/SEARCH.md、artifacts/search-summary.json。

最終source f1c8a2f、smoke-20261115-071815はCPU0/1thread/16GiB、各5秒で8問×4solverを全て新規測定。32run全時間制限、証明0。下表は検証済み最小化主解 (—は主解なし/不合格)。

|問題|Rust|HiGHS|SCIP|GLPK|
|---|---:|---:|---:|---:|
|pk1|19|—|19|21|
|mad|0.3766|0.208|0.4208|0.3512|
|markshare_4_0|2|5|5|8|
|30n20b8|—|—|—|—|
|50v-10|3777.6|3629.31|6799.66|4741.99|
|binkar10_1|6920.29|6751.21|6749.29|6846.46|
|gmu-35-50|-2.53244e+06|-2.60544e+06|-2.60376e+06|-2.39682e+06|
|gmu-35-40|-1.86047e+06|-2.40543e+06|-2.4027e+06|-2.3011e+06|

返却28解はGMP絶対27合格/1不合格/未検証0、公式相対28合格。HiGHS pk1 native約15は既知の絶対違反1.000000008e-6により除外し、Rust7解は両検査すべて合格。gmu50でRustはGLPKの主解を上回るがHiGHS/SCIPには及ばず、gmu40でも大幅な不足が残る。dual/共通gap/全失敗/検証の詳細はartifacts/search-default-summary.json。

86 Rustテスト、best/hybrid各1000小問+200中規模、probe有効と最終既定各1000+200は失敗0。80二値MILP (min/max各40問) ×3探索を独立整数全列挙で照合し、打切り時の下界と局所証明を使わない回帰も追加。07:21 UTC baseline保存解35件の別GMP監査は34合格/既知invalid1/未検証0。最適化snapshotと共有依存は不変。
