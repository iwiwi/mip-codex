# 作業記録

## 2026-11-15: 初期基盤
- 添付指示を読み、空の専用リポジトリから開始。Neoverse-V2 64コア/247 GiBを確認。
- 公式MIPLIB Benchmark v2の240問を取得。data/manifest.jsonに各圧縮入力とアーカイブのSHA256、固定分割を記録。取得終了code 0、artifacts/fetch-miplib.log。
- Rustで二段階密表単体法を自作 (min、非負変数、<=行)。Bland選択、人工変数除去、数値異常/反復制限、元LP残差確認。
- cargo test: 8件成功。最適、実行不能、非有界、負RHS、冗長行、空問題、不正入力、反復制限。
- 採用理由: 小規模の参照基盤として有用。大規模用性能や最適性の厳密証明を主張しない。既存コードの移植なし。
- 未完了: MPS読込、MILP、独立Python検査器、比較ソルバー導入、版固定、ランナー、ランダム差分検証、疎LP。最終比較は未実行。
- 次: MPS/モデル/CLI/独立検査器を整備し、合成問題でLP差分検証。その後分枝限定を実装。密表方式はMIPLIB規模では不十分なので疎改訂単体法を優先的に開発。
- 全体目標は未達成。期限まで改善を継続。

## 2026-11-15: MPS、MILP参照実装と検証経路
- 疎な元モデル、MPS/plain+gzip reader、inspect/dump/solve CLIを実装。RANGES、自由変数、目的方向・定数、整数marker/境界を扱う。
- 全240問を独立HiGHS readerと行列・行/列境界・目的関数・整数性・名前まで照合。初回momentum1だけ7微小係数で不一致。原因はHiGHSの既定small_matrix_value=1e-9。1e-12へ変更して再照合し一致。初回失敗ログも保存、集計artifacts/mps-reader-summary.jsonは240/240一致。
- Rustの密表LP上に深さ優先の分枝限定を実装。整数変数境界丸め、一般境界→非負変数変換、解と双対境界・gap・終了状態をJSONで保存。LPの最適/実行不能判定に双対/Farkas乗数の数値残差チェックを追加。LP非有界からMILP非有界を断言しない。
- 独立Python検査器は元MPSを別readerで読込み、全元制約・変数境界・整数性・目的値を再計算。検査器の意図的な違反検出テストを追加。
- cargo test 17件成功。固定seedの小規模LP/MILP 1000問をHiGHSと比較し失敗0、純整数では全列挙にも一致。各問題hash、生成コード、代表解、元MPS、検証手順を保存。
- 比較ソルバー導入: HiGHS 1.16.0、SCIP 10.0.2、GLPK 5.0。比較専用Python workerと1CPU/16GiBの逐次smoke測定を実装。
- 5秒・開発3問のsmoke結果 (全12runで最適性証明0): pk1 primal Rust 46 / HiGHS約15 / SCIP19 / GLPK18、mad Rust解なし / 0.208 / 0.386 / 0.1712、markshare_4_0 Rust約20 / 約5 / 約5 / 7。返却された11解は全て独立検査合格。性能優位は未達成、初期実装の解品質不足が明確。
- 測定はartifacts/smoke-20261115-023707に全コマンド・実ログ・終了コード・資源・解・検査結果・設定を保存。初回開発smokeはdirty=trueで実施しソース/バイナリhashを添付。今後は測定前commitを原則とする。
- cargo clippyは未導入で失敗 (ソース警告ではない)。通常のbuild/testは成功。導入後に実行予定。
- 次: 疎改訂単体法と基底更新/再利用を実装し大規模入力を扱う。現在の密表は推定256MiB上限、再最適化も毎ノード二段階からで競争力不足。伝播/ヒューリスティックの強化と評価ランナーの全240問対応・公平性の監査も必要。

## 2026-11-15: 疎改訂単体法 (実装・検証段階)
- 有界変数の自作改訂primal simplexを追加。疎列形式、eta基底更新、50更新ごとの疎LU再分解、上下限flip、二段階法、元行列の主/双対残差・非有界ray検査。
- faer 0.24.4 (MIT)を汎用疎LU/線形方程式に限定して使用。最適化アルゴリズム委譲はなし。
- 等式/範囲制約を1行化し、有限変数境界を追加行にしない。密表参照エンジンも切替可能。
- 基本テスト25件成功。小規模1000問のHiGHS/全列挙検証失敗0。追加の中規模根LP200問はdense/revised/HiGHSが一致し失敗0。根LPの検査はノード制限1を使い、双対境界を照合。
- 性能実験はこの実装commit後に実施する。ノード間基底再利用はまだ未実装。次にMIPLIB開発用の同条件比較で採否を判断する。
- 初回MIPLIB比較artifacts/smoke-20261115-024834: pk1の探索ノード密表471/疎12190、madは5/11432。疎版はmadでも検証済み解0.4938を得た。markshare_4_0は疎版が既定10万ノード制限に到達。最適性証明は全runで0。
- この結果から既定ノード/LP反復上限をusize::MAXに変更し、通常は時間/メモリ制限で比較する。CLIで明示ノード上限は引き続き指定可能。時間まで探索する修正版を再測定する。
- 非計測の密表LP統計は0ではなくnullで出力。benchmarkのdirty判定を生成ログではなくソース領域に限定。
- 修正版の最終smokeはartifacts/smoke-20261115-025135 (commit ff6c959、ソースclean)。5秒/1CPU/16GiBでdense/revised/HiGHS/SCIP/GLPKを比較。pk1 primal 46/29/約15/19/18、mad 解なし/0.4938/0.208/0.386/0.1712、markshare_4_0 約20/4/約5/約5/7。全15runで証明0、返却14解は全て独立検査合格。
- 大きめの開発3問も5秒比較 (artifacts/smoke-20261115-025252)。30n20b8は密表resource_limitに対し疎版は双対1.56640764558、74ノード、RSS約57.4MiB。50v-10は疎版双対2879.06568685/713ノード、binkar10_1は6637.18802694/70ノード。密表2問は根LP中の時間切れ。両方式とも3問にprimalなし。
- 上記6問の根LPをHiGHS relaxationと照合し全件一致。最終基本テストは26件成功。疎エンジンを既定として採用し密表も参照用に保存。全体優位性は未達成。
- 次の最優先: 全240問の長時間比較を期限に間に合わせるため、再開可能な8枠の評価ランナーと長時間ジョブの監視を整備。比較3ソルバーの3600秒baselineを早期開始する。8枠で3ソルバー×240問×1時間の最悪約90時間、Rust最終評価も約30時間なので今から日程を管理する。その上でノード間基底再利用/dual simplex、伝播、ヒューリスティックを強化。

## 2026-11-15: 長時間評価ランナー
- protocol v1をdocs/EVALUATION_RUNNER.mdに固定。各3600秒/1thread/8枠CPU8..15。cgroupでケースの全子プロセス合計16GiB、swap禁止、CPU1コア、AS上限16GiB。最適化・検査・ラッパーも同じcgroup。
- immutable snapshot、git/source/library/input/binary SHA、各stageのPID/start_ticks/PGID/boot_id、stdout/stderr、終了状態、watchdog、資源を記録する再開可能runnerを実装。グローバルlockで2coordinatorを禁止。終了済みケースは失敗も再実行しない。
- 比較専用GLPK C driver (GPL-3.0-or-later)を追加。公式testscriptに合わせcuts/FPを有効化。丸められたログでなくAPIの境界を1秒までの観測頻度で保存。GLPKの目的定数規約差を合成例で確認し比較側で正規化。Rustとはリンク/呼出しなし。
- MIPLIB公式GMP solution checkerをMITライセンス付きでvendor。既存の絶対1e-6 Python primaryと、GMP/相対linear1e-6/int1e-6の補助検査を両方保存。目的定数/短すぎるfree形式の一部は公式checkerで未対応なのでunverifiedと誤答を区別。
- 全240問に非ゼロ目的定数RHSとOBJSENSEがないことを監査 (artifacts/mps-extension-audit.json)。
- 初期統合12run (3合成問題×4solver) は8検証済最適/4実行不能で一致。返却8解は2checker双方に合格。
- watchdog/生存stage回収の2テスト成功。実coordinatorを中断・再開しsolver PIDとlaunch hashが不変、完了後再開でもresult hash不変。2coordinator同時実行はlockで拒否。別cgroup64MiBで実OOMを誘発し群全体killをkernel memory.eventsで確認。
- baselineが動いている間は共有.venv/GLPK等のライブラリを更新しない。重い開発作業にはtaskset -c 32-63を使い、CPU8..15を空ける。次にclean commitから全240×3のbaselineを開始し、coordinatorの生存を実確認する。
- clean source commit c431aa9の最終統合検査を12run実施し、8検証済最適/4実行不能、返却8解はPython/GMP双方に合格 (experiments/runner-clean-smoke)。
- 全240問×3solver=720ケースのbaseline-v1-20261115を2026-11-15 UTCに開始。coordinator PID23052のstart_ticksと8stageの実プロセス/CPU affinity8..15/各16GiBをkernelから確認。最初の4ケースはprimary検証済最適で終了、エラー0 (保存時点)。
- baselineはartifacts/runs/baseline-v1-20261115内の固定snapshotで稼働。configと進捗はexperiments/baseline-v1-20261115へ保存。次ターンは必ず現在の/procとstatusを再確認し、ファイルの存在だけで生存を判断しない。停止時も動いている同一stageを再起動しない。
- 次: baselineを監視しつつ、ノード間基底再利用/dual simplexと伝播・ヒューリスティックを開発。CPU8..15と共有.venv/GLPK等の変更を避ける。全結果の参照値監査・集計・圧縮公開は未完了。

## 2026-11-15: ノード間基底再利用 (実装・検証段階)
- baselineは実プロセスとstatusで再確認し9/720完了、エラー0、coordinator生存。重い開発/テストはtaskset32..63で実施。
- 親の拡大行列 (slack/人工列を含む) をArcで保持すれば、人工列番号を全ノードでcanonical化せずに再利用できると判断。元列/費用/行sense/slack幅を完全比較し、構造不一致はcoldへ戻す。LU factorとeta列も共有し、兄弟が親状態を書き換えない。
- bounded dual simplexとbound-flipping ratio testをRustで自作。元行列の主/双対残差を検査し、候補不足時は全box境界との行矛盾を検査。確証不足/数値異常は二段階primalへfallback、時間/反復制限時は再試行しない。
- 小規模1000問でHiGHS/純整数全列挙と一致。追加の植込み実行可能点を持つ中規模200問もHiGHSと一致 (全列挙は領域50万点以下のみ)。RHS/上限を3000回変更する独立cold差分検証も成功。
- 現時点の基本テストは35件、合成根LP200問・開発MIPLIB根LP6問も照合成功。warm/coldの既定はまだcoldとして、commit固定後に性能比較し採否を判断する。
- 数学的参考: Huangfu/Hallのdual simplex説明とKoberstein2005のBFRT。既存solverのコード移植なし。Harris比率検定/コスト摂動/steepest edgeはまだ未実装。
- 初回5秒ablation (source554a057、artifacts/smoke-20261115-034804): pk1 cold29/warm27、mad cold0.4938/warm解なし、markshare_4_0 cold4/warm12。warmはmadで3ノード/約29.7万dual pivot、markshareで約201.6万dual pivotと退化による停滞が明確。採用せず修正する。
- 原因の一つはdual退化判定にprimal thetaを使っていたこと。dual betaで判定し、beta=0が続く際は直ちにBlandの行/列選択へ切替。さらに構造によるwarm pivot上限max(64,4*m)でcold fallbackし、時間を浪費するcacheを使い続けない。問題名による閾値は導入しない。
- 退化修正後の5秒ablation (source c745f31、artifacts/smoke-20261115-035058): pk1 cold29/warm20、mad 0.4938/0.2576、markshare_4_0 4/約2。3問とも改善、全返却解は独立検査に合格。warmノード41788/22291/365880、cold11722/10966/321525。初回の失敗実験も保持。
- 既定をwarm有効へ変更。最終1000小問題+200中規模のHiGHS比較は失敗0。35基本テストには目的が一定でdual beta=0が続く3000変更のケースも加え、warm pivot予算を超えて停滞しないことを検査。
- この実装commit後に、現GLPK cuts/FP有効の比較3solverを含めた最終smokeと、大きめ3問のwarm/coldを取り直す。
- 最終比較source17c5872はartifacts/smoke-20261115-035415 (cold/warm/HiGHS/SCIP/GLPK、各5秒CPU0) と035532 (大きめ3問cold/warm)。pk1 primal29/約20/約15/19/21、mad0.4938/0.2576/0.208/0.4208/0.3512、markshare4/約2/約5/約5/8。GLPKは現行cuts/FP有効。小規模15runで証明0。
- 大きめ: 50v-10はwarmのみ8244.969954、binkar10_1はwarmのみ6944.740016を得た。30n20b8は両方primalなし/同じdualで、ノードcold71/warm51と改善せず。この結果も省略しない。
- 最終21run返却17解は元MPSへのPython絶対検査・公式GMP検査とも全合格。解・元入力hash・検証コマンド・結果を保存 (artifacts/warm-final-summary.json)。ノード比を速度倍率とは主張しない。
- baselineは04:02 UTC時点で11/720完了、全11がprimary検証済最適、エラー0、coordinator kernel生存。次ターンは現在の状態を再確認する。
- 次: 元制約の安全なbound propagation/整数丸めによるLP呼出し削減と、初期実行可能解のdiving/repairを実装する。カットとbranching改善も必要。稼働baselineのCPU8..15・共有環境を保護し、検証用群の結果を改善判断には使わない。

## 2026-11-15: 伝播と検査器の精度監査 (進行中)
- 制約からの区間伝播をRustで実装中。積・和・差・除算を外向き丸めし、整数境界を丸める。既定は無効のまま、42 Rustテストと1000小問題の比較を通過。初回の伝播予算不足は新たに固定された変数の隣接行分だけ予算を補充して修正。失敗ログも保存。
- 浮動小数点の元制約検査に反例を発見。10^8×10^8+1≤10^16 は違反1だが従来のfsum検査が合格する。artifacts/validation-cancellation*に再現データを保存。過去の浮動小数点検査合格を厳密検査合格とは扱わない。
- 公式GMP checkerに別マクロ/別バイナリの絶対誤差版を追加。元十進数を有理数として検査し、反例を正しく棄却。目的定数RHSと短いfree形式名にも対応。通常版の相対検査は補助として残す。主検査不能時はvalid=Noneとし、浮動小数点検査へfail-openしない。
- 04:36 UTC時点のbaselineは21/720完了、エラー0、coordinator生存。15件の旧primary検証済最適という集計には新精度監査が未適用。最適化/共有環境/snapshotは変更せず、保存解を同じ新基準で再監査する。
- 次: 新有理数primaryで1000+200問題を再照合、保存解の再監査、伝播のon/off性能比較、既定採否。全240×3評価とRust競争力改善は未完了。
- 新primaryによる1000小問題+200中規模、合成根LP200+開発根LP6、44 Rustテスト、検査器2テストが成功。十進解の末尾桁をfloatへ丸めず、Decimalからcheckerへ渡す回帰も追加。
- fd31fceの伝播ablationは6問題×on/off各5秒。小3問で利益なし/mad悪化。大3問では30n20b8のdual強化、binkarのprimal改善、50v-10のprimal悪化。既定無効を維持。性能結果を都合よく省略しない。
- 過去PR6返却17解+今回10解を新primaryと公式相対検査で再監査。新primary26/27、公式27/27。HiGHS pk1の行違反が1.000000008e-6で絶対1e-6を超えるため新基準不合格。過去の「全解絶対検査合格」を訂正。Rust解の不合格はなし。元runは変更せず別auditディレクトリへ保存。
- baseline保存解19件は有理数絶対検査で全合格。監査対象は完了かつ解ありのみで、解なし/未完了を成功扱いしない。

## 2026-11-15: 丸め・座標修復ヒューリスティック (実装/比較中)
- PR7をmainへmerge (b85ed21)。伝播は既定無効。有理数絶対primary、公式相対補助、浮動小数点診断という役割をPLANにも反映。
- primal-repair branchでLP点の整数丸めと、正規化した制約違反を減らす座標変更を実装。最初は最隣接丸め、残り3回は固定seedの確率丸め。継続的な改善がない場合は打切り。既存solver/最適化ライブラリの利用なし。
- root最大50ms、以降256ノード毎に最大1ms、作業量も構造由来予算で制限。候補を得ても下界・実行不能判定は変更しない。通常検査に加え各行の外向き区間が許容内にある場合だけ主解候補とする。大きい桁の等式で保守的に候補を棄却することはある。
- 48 Rustテスト通過。新GMP primary付き1000小問題のHiGHS/全列挙照合は失敗0、80問題で修復からのincumbent更新あり。rootで主解を得ても未処理ノードの下界を変えない回帰、連続変数修復、桁落ちの誤候補拒否、期限を検査。
- source2c4142fを固定してon/off各5秒の開発群比較を開始。中規模200問題照合も実行中。効果確認まではMIP_PRIMAL_REPAIR既定無効。baselineは04:42 UTCで21/720完了、エラー0、kernel coordinator生存。
- 座標修復だけのsource2c4142fは6問題で検査済み。50v-10の5秒primalが8244.969954→3777.599989へ改善、小3問とbinkarの最終primalは同じ。30n20b8は両者主解なし。10解は新GMP/公式相対双方合格。
- 連続変数が連動する場合、単一座標の違反総量が下がらず止まる。整数を最隣接値に固定した補助LPを自作Rustのwarm revised simplexで解く修復を追加。補助LPのdual/infeasible/制限は木の証明に使わず、検査に合格した主解だけ利用。LP呼出しとpivotは全体統計にも含める。新しい下界を捏造しないroot回帰を追加。
- source6a6f961のablationも50v-10で同じ3777.599989を得た。初回主解秒はpk1 0.00842→0.000408、mad0.3907→0.00108、50v-10 0.5957→0.00666。markshare/binkar/30n20b8には明確な改善なし。全12run制限、証明0、返却10解は双方検査合格。
- 49 Rustテスト、補助LP付き1000小問題/200中規模照合も失敗0。初期主解と50v-10の品質改善から既定有効へ変更 (e6a83c2)。新規比較は6問×Rust/HiGHS/SCIP/GLPK各5秒で進行中。smoke中のbinary hash変更を検知するguardも追加し、意図せず測定中にbuildを切り替えた結果を混ぜない。
- audit_smoke_solutions.pyを追加し、元run不変のまま両検査とhash/コマンドを別ディレクトリへ保存。初回手動指定で実在しないrun名を入力して監査が失敗したため、run/experiment.jsonの存在を開始時に検査するよう修正し、実際のrun名で再実行した。誤った監査結果は採用していない。
- 次の分枝・探索仮説はdocs/BRANCHING_PLAN.mdへ保存。SCIP公式資料の数式/設計を参照するが、コード移植なし、順位変更と証明用下界を分離する。
- 最終source e6a83c2: artifacts/smoke-20261115-044926、6問×4solver各5秒CPU0。Rustの50v-10 primal3777.599989はSCIP6799.659986/GLPK4741.989989より良いがHiGHS3629.309987より悪く、dualは全比較solverより弱い。binkarはprimal/dualとも他3solverに及ばない。全体優位未達成。全24run制限、証明0。
- 20返却解を新primary/公式相対で再監査し19/20、20/20。HiGHS pk1は厳密行違反1.000000008e-6で今回も不合格。Rust5解は双方合格。rawと検証済primal、共通gapをrepair-final-summary.jsonへ分離保存。既定有効の最終1000+200照合と49 Rustテストも通過。
- 次: この主解改善をPR/mergeし、方向別擬似コストと探索順、次に予算付きstrong branching/カットを進める。固定baselineは変更せず、実PID生存と進捗を再確認する。

## 2026-11-15: 擬似コスト分枝と子探索順 (比較中)
- PR8 main merge2c8ffb5を起点にpseudocost-branching branch。前ターンは実装/比較/mergeという進捗。baselineは04:54 UTCで21/720完了、エラー0、coordinator kernel生存を確認し継続。
- 子ノードへ分枝変数/方向/距離/親LP目的を保存し、実際に解けた子LPの目的増分から方向別擬似コストを学習。2サンプル相当の全体平均priorで少数観測を平滑化し、両方向の予測増分の積 (logで比較) で変数順位を決める。全体コストがゼロなら従来の最大fractionalへ戻す。学習値は順位だけであり、下界・棄却には一切使わない。
- floor側先行/down、整数に近い側/nearest、予測増分が小さい側/estimateを独立に切替。MIP_BRANCHING=fractional|pseudocost、MIP_CHILD_ORDER=down|nearest|estimate。既定はまだfractional/downで採用前比較を行う。
- 52 Rustテスト成功。学習が変数順位を変えるケース、ゼロコストfallback、全6rule/order組合せのmin/max/offsetを確認。pseudocost/estimateの1000小問題+200中規模を新GMP primary/HiGHS/全列挙で照合し失敗0。他の新探索順も別途1000問題を検証中。
- source6d138a2で6問×4方式の各5秒比較を開始。LP warm、主解修復有効、伝播無効を共通に固定。失敗/悪化も省略せず、少数問題の順位だけで全体優位と主張しない。
- smokeスクリプトの未使用だったcold/warm切替経路に、以前の文字列置換で環境変数名がtuple keyになったバグを発見し修正。過去に実際に測ったprop/repair系の経路には影響なし。CPU1/0秒のcold/warm実コマンドで環境値0/1と正常なJSON出力を確認 (smoke-20261115-045616)。0秒runは性能結果に数えない。
- source6d138a2の6問×4方式 (CPU0各5秒) はpseudocost/downがmad0.2576→0.2218、binkar6944.740016→6904.630026へ改善。他4問の主解/下界は同等。nearest/estimateはpk1改善とmad/binkar悪化が混在。
- 測定前に開発12問を圧縮サイズ/固定seedで抽出しCPU1各2秒×4方式で追加比較。pseudocost/downはgen-ip002改善、他の主解5問は同等。全18問中の主解あり11問で3改善・8同等から擬似コストを既定化し、floor側先行を維持 (2f505c7)。nearest/estimateは任意実験のみ。
- 全72runはtime_limit64/numerical_error8、証明0。返却44解はGMP絶対/公式相対とも全合格。新3探索方式それぞれ1000小問題、estimate/採用downの各200中規模、52 Rustテストを通過。既定変更後の最終1000+200も失敗0。
- gmu-35-50/35-40は全4方式で根LP数値エラー。旧fractionalも失敗するため新順位だけの問題ではない。gmu-35-50 revisedは176pivot/5refactorで数値エラー、denseは30秒制限。独立HiGHS根LPは両者optimal。次にnumerical_errorの箇所を診断し、残差/安定化を調べる。数値エラーをinfeasibleと誤宣言していない。
- 05:00 UTC baseline22/720、kernel coordinator生存。cbs-cta/GLPKがnative optimal0を報告したが、行c5862の違反1.44935807712805によりinvalid_solution。旧絶対・新GMP絶対・公式相対すべて不合格。GLPK自身の保存raw再読込/KKTでもPB最大違反1.44935807712805、rowupper0を確認。native生解とhash/監査を保存し、証明や検証済primalに数えない。稼働snapshotや共有solverは変更せず残りを継続。

## 2026-11-15: 根LP数値安定化 (検証/性能測定中)
- 前ターンはPR9の実装・比較・mergeという進捗。main b63bd91からnumerical-stability branch。baselineは05:04 UTC時点22/720、kernel coordinator生存、既知invalid_solution1を維持して測定継続。
- MIP_LP_TRACE=1の失敗位置出力を追加してgmu-35-50を再現。値-1.1229e-13/pivot4.1523e-6の比率をFEASと比較する単位の誤りを特定。値の単位で検査する修正後は進んだが355pivotでLU特異となり、Harris二段階検定を追加した。543pivot/約0.02秒で根LPを解き、dual-2608070.3157430096がHiGHSと一致。gmu-35-40も一致。原/中間失敗ログを保存。
- 価格付け監査で微小目的係数の真の反例を追加。min y-1e-12*x, x=z, x/z>=0, y∈[0,1]は非有界だが旧mainはoptimal0を報告した。主解0の実行可能性検査だけでは見つからない。相対価格付けとray目的改善判定を実装し、1e-12/1e-18の係数、大きい有限幅をregressionに追加。初回はray側の固定閾値が残ってテスト失敗、修正後は乗数ノイズを価格として拾ってwarm/coldテストが反復上限になった。noise floorを加えて再実行し成功。両失敗ログを保持。
- スケールを変えた100 LPではrevisedは全通過、denseが27問失敗。denseの行/目的正規化とHarris、元単位へdual/Farkasを戻す処理を実装。行正規化前の試作がfixture8をfalse infeasibleとしたことをregressionで検出し不採用。最終版では2fixture回帰、通常200LP、scaled100LP (両エンジン) が成功。許容誤差は緩めず、比較誤差も目的倍率で正規化した。詳細はdocs/NUMERICAL_STABILITY.md。
- 60 Rustテスト、revisedの1000小問題+200中規模、開発8根LP比較が成功。最終ソース固定後に8開発問×Rust/HiGHS/SCIP/GLPK各5秒を測定する。新規のスケール試験の失敗と中間試作をすべて保持。
- 05:17 UTC baseline23/720、旧primary最適16、invalid_solution1、kernel coordinator生存。snapshot/共有依存/CPU8..15は変更しない。LP列スケーリング等の更なる安定化と、strong branching/カットによる下界強化は次の候補。
- 最終source1e093b8の全32runは時間制限、証明0。gmu2問で根LP数値エラーから主解とdualを返す状態へ改善したが、主解は他3solverに大幅に及ばない。mad0.2218→0.3766、binkar6904.630026→6920.290013と旧方式より悪化もあり。正しさ修正を採用し、性能悪化を隠さない。
- 返却28解のGMP絶対27/28・公式相対28/28。HiGHS pk1は既知の閾値超過で不合格、Rust7解は双方合格。全rawと検証済primal/共通gapをstability-final-summary.jsonへ保存。
- 旧両エンジンのoptimal0を反証するx=z=1e12,y=0, objective-1の点を元MPSにGMP絶対検査し合格。before/afterの両engine出力、source/binary/input hashをstability-source-provenance.json等へ保存。これを単なる速度改善と扱わない。
- 最終両エンジンの1000+200照合は各失敗0、60 Rustテストも成功。通常根LP200/scaled100は全通過。初回scaled100の27失敗と中間のfalse infeasible/反復上限を保持。
- 05:24 UTC baseline24/720、旧primary最適16、invalid_solution1、kernel coordinator生存。新GMP primaryで保存解21件を再監査し20合格/1不合格、未検証0。最適化snapshotと共有依存は変更せず継続。
- 次: LP列/目的のスケーリング、極端な係数/幅と非有界rayの検証を増やす。gmuの主解品質から、LP潜り/feasibility pump等の一般的主解探索も必要。次に予算付きstrong branchingとカットで下界を強化する。専用問題名による分岐は禁止。

## 2026-11-15: LP列・行スケーリング (PR11でmerge済み)
- 前ターンはPR10 mergeという進捗。main c2e97b8からlp-column-scaling branch。旧疎LPは min -x, 1e-12*x<=1 をunbounded_relaxationと誤判定。列を2の冪で正規化してoptimal -1e12へ訂正。列だけではx-z<=0を加えた結合例がなお誤非有界となり、列の後に小さい行も正規化して両例を解いた。before/intermediate/after出力をartifacts/column-scaling-*へ保存。
- 変換は係数/目的/幅のroundtripを検査し、underflow/overflowを黙って0/無限大へしない。元Problemで主点/dual、Farkas候補、rayを再検査する。warm cacheは正規化後の構造で一貫させる。Statsにscaled_columns/rows追加。
- 実行不能候補の外向き区間がゼロをまたぐ場合、unbounded列の積和符号を固定長整数で正確に計算する。f64全指数域の積和符号はPython Fractionで作った501fixture (大きい積2000個の相殺+最小subnormal積を含む) と一致。rayの元行/目的符号も正確に検査する。厳密対象は元Problemのf64数値で、元MPS十進係数の有理数最適性証明という主張ではない。
- 変数単位を2^-30..2^30、行を2^-12..2^12で変えた100根LPはrevised全通過、返却LP点も元MPS GMP検査合格。参照HiGHSは変換前の同値モデルで解き、tiny係数のreader dropを避ける。数学的に既知のfractional knapsackを125単位系で検査する回帰も通過。
- denseの同100問は初回83 numerical_error。人工変数後処理を既存FEASへ揃える、元境界への点の投影と元行/dual再検査、元行列からのbasis再構築で50 numerical_errorまで減少。共通RHS単位を更に正規化する試作は79失敗へ増えたので棄却し元へ戻した。誤答を成功に数えず失敗ログ/入力は保持。通常回帰は全通過。現在のdense extreme-unitsは未解決で、revisedの成功と混ぜない。
- 非有界候補はrayだけでなく元単位の基点も検査するよう変更済み。現在は小pivotが有限上限を規定する近平行制約の追加回帰を検討。最終1000+200、通常/scaled根LPとMIPLIB8根LP、性能比較・PRは未完了。
- baselineは05:51 UTC時点29/720、既知invalid_solution1、kernel coordinator生存。固定snapshot/共有依存/CPU8..15は変更しない。

- 最終source4bff42c: 73 Rustテスト、両エンジン各1000+200、通常200/scaled100/開発8根LP成功。近平行有限LPを微小pivot再試行で解き、長い正確な十進係数を新GMP絶対検査器で確認。検査器32文字制限とPython診断のO(m²)行境界取得を修正、NumPy試作のJSON失敗は保存。3 Python回帰成功。
- 極端なunits100問はrevised全成功・元MPS GMP合格、denseは50 numerical_errorを残す。性能8問は全時間制限/証明0、返却7解双方検査合格。全8問の変換数0なので性能差をスケーリング効果と主張しない。比較3solverは前回同資源runを再利用。
- 06:48 UTC baseline39/720、旧primary最適18、errors2、kernel coordinator生存。errorsはcbs-cta/GLPK invalidとbuildingenergy/HiGHSの旧検査600秒時間切れ。新検査器監査33件は32合格/既知invalid1、未検証0。旧監査の既知O(m²)による停止と失敗試作も保存。
- 次は一般的な予算付きLP潜りによる主解改善。gmu主解の大幅な不足を検証し、強分枝/カットの下界改善へ進む。固定全件baselineを継続し、期限内のRust全240問比較時間を確保する。

## 2026-11-15: 予算付き根LP潜り (PR12で採用)
- 前ターンはPR11 merge (7b6b0cf) の進捗。lp-diving branchで根LPから整数の近い側へ境界を逐次絞る主解ヒューリスティックを実装 (f06522d)。最も整数に近いfractional変数を選び、局所LP infeasibleなら同じ親の反対側を一度試す。96局所LP、各512pivot、全体時間5%か0.25秒の小さい方。既定無効から比較する。
- 根ノードのwarm基底を再利用し、局所LPの戻り値は主解候補だけに使用。下界、擬似コスト、探索木、最適/実行不能判定に反映しない。整数を丸めた候補は元Modelの境界/整数性/制約と外向き区間で検査し、更に元MPS GMPで独立検証する。局所LP回数/反転/失敗/制限/主解/時間を出力。
- 77 Rustテスト成功。新4回帰は根下界を変えず主解発見、失敗側から反転、両側局所失敗でも全体infeasibleを宣言しない、期限0で実行しないことを検査。MIP_LP_DIVING=1の1000小問題+200中規模 (HiGHS比較と純整数全列挙、返却解GMP検査) は失敗0。
- 既存8開発問×各5秒CPU0でon/offを比較中。以前選んだ追加開発12問から重複gmu2問を除いた10問×各2秒CPU1でもon/off比較中。holdoutを選択/調整に使わない。最終の実測・採用判断・PRは未完了。
- 06:53 UTC別監査: baseline保存解33件32合格/既知invalid1/未検証0。旧検査600秒時間切れのbuildingenergy/HiGHSも新GMPでは合格し、native最適化を再実行していない。稼働snapshotや共有依存は不変。

- 初回f06522dは18問中pk1のみ改善、返却26解双方検査合格。ただし既存修復より先の潜りが初解を遅らせた。29f548eで修復先行・整数一括固定・256LP/4096pivotへ変更し、bppc4-08も86→72へ改善。gmuは一括固定で局所実行不能となり未改善。
- 9e4d19aは一括固定が失敗した場合、共通期限内で固定を減らして再試行。各試行256LP、合計壁時計は5%/0.25秒。18問の終了主解は2改善/16同等 (主解なし5問含む)、全36run時間制限/証明0、返却26解は両独立検査合格。gmuの大幅な不足は未解決。新bpp比較のHiGHS57/SCIP58/GLPK64にもRust72は及ばない。
- 78 Rustテスト、候補ごとの1000+200比較は失敗0。局所固定で実行可能な整数解を失う小例を追加し、根へ戻る再試行と探索木を捨てないことを検査。期待値が局所失敗2→3になるテスト失敗と、既定化で旧修復単独テストへ潜りが混入した失敗は保存し、機能を明示分離して検査する。
- 2件改善と残りの主解同等から既定化。最終既定後の1000+200と8問smoke/独立監査を実施してPRへ。次は根LP安定化後に未比較の子ノードnearest/estimate探索順を再評価し、gmu等の主解品質を改善できるか調べる。さらに一般的な主解探索、強分枝/カットへ進む。

- 既定fb94450の最終1000+200は失敗0、78 Rust回帰も成功。8問5秒の最終smokeは全時間制限/証明0、返却7解はGMP絶対/公式相対とも合格。gmu-35-40は-933357.2996から-923666.5855へ悪化し、打切り時刻の変動と潜り消費の影響を調べるためCPU0でon/off各3回を追加測定する。少数の改善だけで悪化を省略しない。
- 07:01 UTC全件baseline41/720、旧primary最適19、errors2、kernel coordinator生存。新GMPで保存解35件を別監査し34合格/既知invalid1/未検証0。snapshot/共有依存は変更しない。

- CPU0/5秒のgmu-35-40 on/off各3反復では、再試行有効が全て-923666.5855、無効が全て-933357.2996。返却6解は両検査合格。約0.15秒の追加消費が主解を悪化させるため、c58d292で再試行を削除し、2改善/16同等だった整数固定版へ戻した。最終78テスト、1000+200とも成功。既定版8問とgmu各3反復をもう一度測定して採用変更を検証する。

採用c58d292の最終smoke-20261115-070316は全8run時間制限・証明0、返却7解は両検査合格。gmu-35-40は-933357.2996へ戻り、CPU0/5秒の追加on/off各3反復 (070356/070407/070417) も全6run同じ-933357.2996。主解を悪化させた再試行を取り除いたことを確認した。78 Rustテストと最終1000+200比較は失敗0。全18問の対照・棄却試作・最終版は別々に保存し、最終値はartifacts/diving-adopted-summary.jsonを参照。

## 2026-11-15: ノード探索戦略 (PR13で採用)
- 前ターンはPR12 merge (506b5bc) の進捗。search-strategies branch。数値安定化後の18開発問で方向down/nearest/estimateを再比較 (8×5秒CPU0、10×2秒CPU1)。sourcebf1b23b (smoke-20261115-070635/070636)。全54run時間制限、証明0、返却37解はGMP絶対/公式相対とも合格。nearestはgmu両問の初解を失い、mad/binkar悪化。estimateはgmu50 -112379.8182→-2540189.7008、gmu40 -933357.2996→-1860472.4551、gen-ip002/pg5_34改善、mad/binkar悪化。全体既定にはまだしない。
- 自作Queueで最良下界と混合探索を追加 (6edb5da)。混合は64popごとにDFS待ちノードをheapへ移し、最小下界からDFSを再開。同じ下界では既存の子優先順を保つLIFO tie。各ノードはstack/heapの一方だけに存在し、終了時下界は両方全ノードを含める。ノードの取出し順以外の枝刈り条件を変えない。
- 83 Rustテスト通過。3Queue回帰と、80二値MILP (最小化/最大化各40問) ×3探索の独立全列挙比較、半整数等式の130ノード打切りで残りノード下界を保持する回帰を追加。新2戦略それぞれ1000小問+200中規模は失敗0。
- depth/down、best/down、hybrid/down、best/estimate、hybrid/estimateを同じ18問で対照測定中。既定はdepth/downを維持。測定結果、失敗/悪化、採用判断、最終確認とPRは未完了。固定baseline snapshot/共有依存/CPU8..15は変更しない。

- 6edb5daの5探索×18問は全90run時間制限/証明0、返却54解はGMP絶対/公式相対とも全合格。最良下界downはpk1 primal17→12/dual0→4.8242、50v primal3777.60→3559.30/dual2879.07→3019.95だが、mad/binkar/markshareの主解が悪化。混合estimateは50v3557.89へ改善、他で悪化。gmu両問はbest/hybridの4モードすべて主解なし、depth/downのみ主解を返す。全て任意実験に留め、主探索の既定depth/downは維持する。
- 主解専用probeを追加。別の探索順の主解を本体へ渡し、局所下界/証明は無視する。自作solveを再帰一段のみ呼び、probe側を必ず無効化。4096局所ノード、全体時間20%または1秒の小さい方、残り全体期限で打ち切る。主探索のノード数とprobeノード数を分け、LP総数/pivot/基底統計にはprobe分も加える。既定無効で対照測定中。
- probeは推定コスト順/DFS、主探索はfloor/DFS。86 Rustテスト、probe有効1000小問+200中規模の比較中。根ノード1の打切りで主解だけを採用し下界不変、probe内のinfeasibleを本体の証明にしない、期限0の停止を回帰化。最終ベンチマーク/監査と採用判断は未完了。

- 主解専用probe対照167816dは18問で3改善/1悪化/14同等。gmu50 -112379.8182→-2532435.2987、gmu40 -933357.2996→-1860472.4551、gen-ip002 -4703.110424→-4706.617100。pk1 17→19へ悪化。全36run時間制限・証明0、返却26解はGMP絶対/公式相対とも全合格。主探索全体をestimateにする場合のmad/binkar悪化を抑え、大きなgmu不足を減らすためprobeを既定化。83+3=86 Rustテスト、既定化後1000+200比較は失敗0。
- 最終sourceはsearch-default-smoke-launch.logに対応するexperiment.json。新規8問×Rust/HiGHS/SCIP/GLPK各5秒CPU0を測定中。参照値を再利用せず更新する。全体優位は未達成。queueの比較はprobe無効での結果であり、probe有効後の組合せはまだ調整していない。

- 最終sourcef1c8a2fの新規8問×4solver各5秒は全32run時間制限・証明0、返却28解GMP絶対27合格/1不合格 (既知HiGHS pk1)/未検証0、公式相対28合格。Rust7解は双方全合格。gmu50Rust -2532435.2987はGLPK -2396821.7722を上回るがHiGHS -2605437.3597/SCIP -2603755.2784に未達。gmu40Rust -1860472.4551も3solverより大幅に劣る。pk1は19で前PR12の17より悪化。詳細はsearch-default-summary.jsonとBENCHMARKの全表。
- 86 Rustテスト、best/hybrid各1000+200、probe有効1000+200、最終既定1000+200は全成功。全試作・失敗/悪化を保存。07:21 UTC保存baseline解35件の新GMP再監査は34合格/既知invalid1/未検証0。snapshot/共有依存は不変。
- 次はprobeで強い主解を持った状態でbest/hybridを再評価 (今回queue比較はprobe無効)。さらに予算付き強分枝・カットで下界を強化する。全240問最終Rust評価は未実施で、期限の時間予算を維持する。

## 2026-11-15: 整数証明付きcoverとMPS重複項 (PR14で採用)
- 前ターンはPR13 merge (833da9e) の進捗。cover-cuts branch。probeを有効にしたまま5探索×18開発問を比較 (smoke-20261115-072502/072504)。全90run時間制限・証明0、返却62解はGMP絶対/公式相対とも合格。gmu両問は全方式がprobe由来の主解を保持したが、それ以上の改善なし。既定depth/downを維持。
- 元MPSの係数/RHS/RANGESが厳密な小整数 (絶対値2^52以下) の行にexact_integralメタデータ追加。原文字列を検査し、f64で整数に丸まっただけの1.000...001を信用しない。二値変数だけの行のupper/lower両側を正のpackingへ変換し、貪欲選択したcoverをi128で検査。元行の最小活動量がRHSを超えることを別関数で検査し、全カットへ元行/side/coverを証明として保存。最大4round×32cut、根時間5%/0.25秒。既定無効。
- Python Fractionで元MPSの十進文字列を独立に読み直すverify_cover_cuts.pyを追加。主解なしの終了でも検査し、不合格/未完了なら下界と全体の最適/実行不能主張を採用しないよう将来snapshot用benchmark_caseとsmokeを修正。稼働snapshotは不変。実行不能を装った偽ソルバーの不正cutがguardで除外される4 Python回帰と、既存stage2回帰が成功。
- 初期18問のcuts on/off (073944/073946) は全36run時間制限、返却26解双方検査合格。実際のcutは全18問0本。pk1の主解19→17の差はroot LP warm経路によるもので、cut効果と主張しない。純二値1000問では477問に2969cut、全て元MPS Fraction検査と整数全列挙/HiGHSに合格。
- 入力監査で重複項1e16+1-1e16の相殺を発見。旧版は元のx<=1を0*x<=1としてoptimal -2,x=2を返し、GMP絶対で行違反1。目的だけの重複項でも誤optimal0 (真opt -2)、-0.1-0.2+0.3では誤unbounded (真opt0) を保存。整数配列でf64積和を一度丸めする関数とPython Fraction624fixtureを追加し、元十進値が丸めで相殺する場合もあるためnum-bigint0.4.8 (MIT OR Apache-2.0、汎用整数演算のみ) で重複十進項を合計してから一度f64へ丸める。LP/MIP最適化は委譲しない。非ゼロの入力/合計が0へunderflowする場合はエラー。
- Source670f15bが最終候補。95 Rustテスト成功。整数重複12順序と十進相殺を回帰化。初期全240 MPS行列照合は全成功、最終版の全240照合が進行中 (cover-final-mps-matrix.jsonl、実プロセス確認してから再開)。最後の最終1,000混合+200中規模、純二値cut on/off各1,000は全成功。
- 純二値の隔離比較 (repair/diving/probeを無効) は同じ1000入力/hash。cut off nodes5320/LP5320/solve時間合計0.106726秒、on nodes3676/LP5424/0.201837秒。nodeは減るが総LPは増え、極小問題の時間は悪化。MIPLIB18問でcut0も合わせ、性能既定化はしない。証明付きの任意機能として保ち、混合行・lifting等は次の課題。
- 07:45 UTC baseline47/720、旧primary最適19、errors3、coordinator生存。新しいerrorはbuildingenergy/SCIPの旧診断600秒時間切れ。保存解40件の新GMP別監査39合格/既知GLPK invalid1/未検証0。snapshot/共有依存/CPU8..15は変更しない。
- 残り: 最終240行列照合と8Rust5秒smokeを確認、証明と原MPS解の最終監査、文書/AI資源量/PR/merge。重複objectiveだけのMPSは公式checkerがRHS省略を拒否するため、独立Fractionの一変数モデル検査で真の最適値とafter点を検査し、元行を含む反例は元MPSのGMPでも確認する。失敗checkerログと最初の監査パス誤りも保持。

- 最終全240 MPS行列/目的/境界/整数性照合は全成功 (cover-final-mps-matrix.jsonl)。96 Rustテストと元十進612/F64合計624fixture、最終1,000混合+200中規模、純二値on/off各1,000も成功。最終既定8Rust各5秒 (075311/source670f15b、cut無効) は全時間制限/証明0、返却7解双方検査合格、主解/下界はPR13同等。比較3solverは前回071815を再利用・今回は未再測定。
- 新機能の採用は任意coverだけ、既定は無効を維持。入力の誤最適/誤非有界の修正は採用。原MPS Fractionの3反例検査は全成功、RHS省略のGMP構文制限ログは未検証として保存し合格に混ぜない。次は混合行・非整数係数・lifting等の一般化または強分枝で、実MIPLIBでも下界を強化する。全240最終Rust比較と全体優位は未達成。

- 08:04 UTC baseline49/720、旧primary最適19、errors3、kernel coordinator生存。PR14で入力の相殺修正と証明付き任意coverを採用。coverは実MIPLIB信号不足と極小問題時間悪化から既定無効を維持。

## 2026-11-15: 混合coverとRANGES端点 (PR15準備)
- mixed-cover-cuts branch。元MPSの係数/行辺/変数境界を包含する外向き区間を保持し、有限な連続・一般整数項をbox最小値で射影。二値部分から弱めたpackingを生成し、原行で全cover変数を強制した最小活動量がRHS上界を超えることを整数または区間演算で再証明する。外部検査は元十進MPSのFractionによる別実装。未知/無限境界は必要な方向で棄却。
- 新しい真の誤最適反例: E行 x、RHS10000000000000001、RANGES -10000000000000000。真の下辺1を旧f64加算が0に消し、旧版optimal0,x0を返す。元MPS GMP絶対検査は違反1 (float診断は誤って合格)。元十進で端点を合計して一度丸め、上下辺をbounded slackで再現できない範囲を二本の行へ分割し、修正版optimal1,x1とGMP合格を確認。beforeバイナリ/入力hashと全ログを保存。
- 試作Python検査器でcolumn boundをrow RHSの変数boundに上書きするバグを追加してしまった。5単体テストが検出。初期混合1000例の468検査失敗と初期smokeのinvalid_cut_proofは保持し、修正後の再実行だけを証明の根拠とする。fixed auditは初期36run/67cut全合格、26主解もGMP/公式双方合格。
- 最初の混合cover比較082521/082522は2主解改善/1悪化/15同等。元のuncut根からも同じ潜り予算で再探索する試作d902641 (083541/083543) は18問すべて同じ主解で追加LP/時間だけ増加し棄却。三角形のcut本数>=3という試験期待も誤りで、有効cut2本でも最適整数点と根下界は正しいため性質を検査する回帰に変更。
- 7982a83は補助repair/divingも有効cutを含む同じLPモデル/基底で解くよう整合化。probeは多様性のため原問題/cut無効を維持し、主解しか本体へ返さない。最新084302/084303: 18問各off/on、全36時間制限/最適証明0。主解2改善/1悪化/15同等。binkar6920.290013→6794.860025、gmu40 -1860472.4551→-1882493.1696、markshare2→4悪化。30n下界1.5664076456→2.0297169811。5問題に計67cut (整数49/区間18、射影49)。元MPS Fraction全67cut有効、26返却解GMP絶対/公式全合格 (mixed-cover-consistent-audit)。
- 現候補101 Rustテスト、混合cut1000/純二値1000/通常小1000/中規模200は失敗0。MPS reader変更e2ee111時点の全240行列照合も失敗0 (以後reader不変)。性能holdoutは調整に使わない。初期のテスト失敗/試作も全保存。
- 2改善と30n下界向上を採用理由としてcover既定有効へ変更中。markshare悪化を隠さない。機能分離試験はcover無効を明記。既定化後の回帰、新規8問×4solver各5秒、最終監査、AI資源/PR/mergeを残作業とする。全体優位・全240 Rust評価は未達成。

- 既定化source7e979b6: 101 Rust回帰、通常小1000/混合cut1000/中規模200が全成功。RANGES反例を新バイナリで再測定しoptimal1/元MPS GMP合格。8問×4solverの新5秒比較はmixed-cover-default-smoke-launch.logで実行中 (CPU0、1thread/16GiB)、完了までバイナリを変更しない。
- 08:49 UTC baseline62/720、旧primary最適24/errors5、coordinator生存。新しいGLPK errorはex10の原MPS reader拒否 (69612行missing row type、driver exit3) とbuildingenergyの3660秒watchdog kill。snapshot改変/隠蔽/再試行はせず全結果へ計上する。保存解51件の新GMP別監査は50合格/既知cbs-cta GLPK不正1/未検証0。buildingenergy HS/SCIPは旧checker時間切れから保存解再監査で合格を回収。

- 最終source7e979b6の新規8問×4solver5秒 (084917) は全32run時間制限/証明0。28返却解GMP絶対27合格/既知HiGHS pk1不正1/未検証0、公式28合格。Rust7解は双方合格、全51cutもFraction有効。主解/下界は最終対照と一致。新規比較を含む全表をBENCHMARKとmixed-cover-default-summary.jsonへ保存。PR15で混合cover既定化とRANGES誤最適修正を採用予定。次は整数行の丸めcutで下界強化を試す。

## 2026-11-15: 一行整数丸めcutの試作 (未採用)
- PR15を434d5c8へmerge。integer-rounding-cuts branch。仮説: 基本coverは2sum(x)<=3の三角形を閉じない場合があるが、一行CG丸めならsum(x)<=1を直接作れる。原係数/RHSが厳密小整数で、全非零項が原境界により非負整数である行だけを対象とする。上/下を向き付け、正整数dで割り、係数とRHSをdiv_euclidでfloor。非負性が必須で、連続/負領域/未知境界を拒否。数学的定義の参考はFabricio OliveiraのLinear Optimisation Notes §10.2 (https://gamma-opt.github.io/linopt-notes/pdfs/linopt-notes.pdf)。外部実装コードは移植しない。
- 証明は原行/side/divisor、coverは空。導出係数/RHSを原整数データから再計算するRust検査と、元十進MPS FractionによるPython別実装を追加。古いcover証明との互換を保持。MIP_ROUNDING_CUTS=0/1で既定無効、probe側は必ず無効。根cut共通4round×32本/5%最大.25秒/work2mの予算内で丸めを先に試す。小整数2/3/4/5/10、原係数の絶対値から最大24候補、gcdを除数候補にする。問題名や既知解は使わない。
- 5追加Rust回帰 (負係数/負RHS floor、下限制約、禁止領域、根最適、250行×全1024整数点の有効性) とPython6回帰が成功、既存101も成功。性能とsynthetic比較はこれから。採用判断前に全cutを原MPSで独立監査する。

- 初期比較pure302cb0a (085639/085644) は丸めcut0本、主解18問同じ。ただし30n下界2.0297169811→1.5664076456に悪化。丸め走査が共通work2mを空費してcover30本を出さなくなったため。36run時間制限、26主解双方検査合格、残る104cover証明も全有効。性能改善として採用しない。丸め用workを50万へ分離、不適用の行/sideを一度調べてから除数走査するよう修正。
- 純整数1000 (非負0..3)、純二値1000、小1000、中規模200は失敗0。Pythonラッパーの一括置換が後段の設定assert箇所にも入りIndentationErrorとなった初回起動ログを保存、修正後に実行。Rustの誤答はこの段階ではなし。
- 非負整数でない項を有限な原境界の最小活動量により射影する拡張を追加。今回は原係数と射影する境界が厳密小整数の場合だけ。i128で寄与を引き、調整後RHSが正確に表現できる範囲外は棄却。独立Fraction checkerも元境界で射影。250混合行×256整数点+連続box両端による検査と、正/負項で使う境界の回帰を追加。原境界不明は棄却。性能再測定と新しい混合合成1000問は未完了。

- 最終候補c456307 (090053/090058) の18問対照が完了。丸めcutはfhnw-binpack4-4の26本 (全て射影付き) のみ。主解も下界も全18問同じ。30n下界を2.0297169811へ保持でき、初期work共食いによる悪化は解消。全36run時間制限/証明0、最終独立監査中。生成数だけを性能改善と扱わず、既定無効を維持する。
- 最終108 Rust回帰とPython7回帰、projected混合1000/二値1000/通常小1000/中規模200は失敗0。新混合profileは連続2列の整数boxと整数係数を持つ。丸め106本を含む全cutを元MPS Fractionで確認。小13本/二値686本の丸めcutも全有効。
- 純二値1000の同じ入力・原解検査でcover有効、repair/diving/probe無効を比較。丸めoff nodes3676/LP5424/合計solve秒0.203432092/cut2969、on nodes3414/LP5144/0.237046726/cut3513 (丸め686)。探索量は減るが時間は悪化。既定化の根拠にしない。
- 次は小数RHSでも原係数が整数であることを区別するメタデータを検討し、丸めの適用行を増やす。また強分枝/信頼度分枝も候補。全240 Rust最終比較を開始する時間予算は維持する。

## 2026-11-15: 根の予算付き強分枝 (試作)
- PR16をedf67c2へmerge。root-strong-branching branch。cutの適用拡大だけでは18問に信号が薄いため、根の分枝選択を試す。最大fractionalの8候補について両側を自作warm LPで試し、各256pivot/全体5%最大.1秒に制限。両側がoptimalで点を原モデル照合できた候補だけ、目的改善のlog積スコアで選ぶ。成功した局所LPから擬似コストを学習するが、局所infeasible/下界/主解を主探索の枝刈り・証明・incumbentへ渡さない。probe側は明示的に無効。分枝前に既存repair/diving/probeを終えるため、既存の根主解機会を先に確保する。
- MIP_STRONG_BRANCHING=0/1、既定無効。LP/ピボット/warm統計は総数へ含め、strong専用候補数/optimal/infeasible/limited/秒/分枝変更数も出す。数学的背景はSCIPのbranch_relpscost.h説明を参照 (https://scipopt.org/scip/doc/html/branch__relpscost_8h.php)、外部のコード移植は行わない。
- 112 Rust回帰が成功。根ノード1で両側infeasibleでも本体node_limit/下界不変、両側optimalでも親下界とincumbent不変、期限0で呼び出し0、80二値問題のmin/max全列挙一致を追加。syntheticと18問性能は未測定。正しさだけで既定化しない。

- sourcef7c2e58の18問on/off (090719/090725) は主解3改善/2悪化/13同等 (微小丸め差は同等)、下界全同じ。timtab1なし→1154004.99999986、gen-ip002 -4706.6170998→-4725.2015367、pg5_34 -13829.2868509→-14019.5666327。悪化: pk1 19→20、gmu40 -1882493.1696→-1860472.4551。全36時間制限/証明0。返却27解は元MPS GMP絶対/公式相対とも全合格、全134cutの証明も有効。主解あり問題が13→14に増えることと3改善から既定採用候補とする。悪化も保存し、全面優位と扱わない。
- 初期strong有効の小1000/二値1000/中規模200は失敗0。strong呼出回数109/251/117、追加LP358/2784/1140で、無効機能を比較しただけではない。最終既定化後の112回帰/合成比較と、新規8問4solver5秒・新たに改善したextra3問4solver2秒を残作業とする。

- 最終既定候補source3e489a5の112回帰と小1000/中規模200は失敗0。新規8問×4solver各5秒CPU0はstrong-default-core-launch.logで実行中 (バイナリは変更しない)。改善extra3問×4solver各2秒CPU1は091022で完了: timtab1 Rust1154005/HiGHS858126/SCIP1031054/GLPKなし、gen-ip002 Rust-4725.2015367/HiGHS-4742.53456975/SCIP-4752.4535304/GLPK-4766.6435686、pg5_34 Rust-14019.5666327/HiGHS-14311.2072606/SCIP-10350.953583/GLPK-13965.0365641。RustはtimtabでGLPK、pg5でSCIP/GLPKを上回るが、HiGHS等へ全体優位ではない。最終監査を進める。
- 次の仮説: RINS型の主解用近傍探索。incumbentと根LPが一致する整数変数だけ固定し、残りを時間/ノード制限付きの自作B&Bで解く。seedを与えて元incumbentを保持し、返却点だけを元モデルで再検査する。局所の証明/下界は本体へ渡さない。再帰helperは全部無効にして一段に制限。既存の丸め潜りは根LPの整数値を全部固定するが、incumbentとの一致に限定することで、少なくとも元incumbentを含む近傍を作れる。実装/検証/性能判断はまだ行っていない。

- 最終新規4solver比較091021/091022は全44run時間制限/証明0。39返却解GMP絶対38合格/既知HiGHS pk1不正1/未検証0、公式39合格。Rust10解は双方全合格、全51cutも元MPS Fraction有効。全値はBENCHMARKとstrong-default-summary.json。誤った監査run名091019を指定した最初の起動失敗ログも保存し、実在091021/091022で再監査した。
- 隔離二値1000問はoff nodes3676/LP5424/.211177373秒、on nodes3418/LP7970/.264023416秒 (局所LP2804)、全参照/全列挙/原MPS検査成功。速度は悪化。主解取得範囲と3改善/2悪化の判断でPR17の既定化を採用予定。次はRINS近傍または選択した子LPのwarm基底だけの再利用を検討する。全240最終Rust評価・全体優位は未達成。

## 2026-11-15: RINS型主解近傍の試作
- PR17を613ec6aへmerge。rins-neighborhood branch。現在のincumbentと根LPの値が1e-6以内で一致する整数変数を固定し、元incumbentをseedとして自作B&Bで近傍だけ解く。少なくとも新規2変数・可変整数の10%以上が一致する場合に限定。根一回、全体5%最大.25秒、1024局所node/各LP4096pivot。元のrepair/diving/probeの後、strong branchingの前。局所repair/diving/probe/strong/cut/伝播/RINSを全部無効にして再帰一段だけにする。nearest/DFS/pseudocostで探索し、主探索の順序は不変。
- 原理の参考: Danna/Rothberg/Le Pape (2005), DOI10.1007/s10107-004-0518-7、およびIBM公式RINS説明 (https://www.ibm.com/docs/en/icos/22.1.2?topic=heuristics-relaxation-induced-neighborhood-search-rins-heuristic)。コード移植や最適化委譲は行わない。近傍モデルの変更は整数境界固定だけ、原Modelは保持する。seedと返却点は元モデルの1e-7/外向き区間検査を通す。局所下界/最適・実行不能宣言は無視し、本体へ主解候補だけ返す。局所LP統計は総数へ加算、局所nodeはrins欄に分離。
- MIP_RINS=0/1、既定無効。共通solveをprivate solve_seededへ分け、通常solveはseedなしのまま。既存probeのLP統計加算も共通関数に整理。117 Rust回帰が成功。近傍がseedを含むこと/元モデル不変、無効seed/不足一致の棄却、局所optimalでも本体node_limitと下界不変、seedは予算0でもglobal proofにならないことを追加。synthetic/18問性能比較はこれから。

- 初期根のみsource7ae5eb6 (091911/091918) は主解2改善/0悪化/16同等。gmu50 -2532435.2987→-2570880.1906、gmu40 -1860472.4551→-2367540.5084。全36時間制限/証明0、28返却解はGMP絶対/公式双方全合格、全134cutも元MPS Fraction有効。内部improvementsカウンタが立つpk1/mad/gen-ip002でも最終主解は対照と同等なので、最終品質の改善件数へ混ぜない。
- 小1000/二値1000/中規模200は失敗0、局所呼出40/171/61回、内部改善1/0/6回。これも両設定が最終最適値を返す合成問題の途中改善であり、最終性能の優位ではない。
- 根ではseedを持たないtimtab1/pg5_34はattempts0だった。incumbentを初めて持った後のfractional nodeで一度だけ試すよう条件を一般化。rootで試した問題は再試行しない。at_nodeを記録し、node3の局所optimalでも本体node_limit/元下界-1.5を維持する回帰を追加。118 Rustテスト後、根のみとの性能差を測定する。未採用。

- 後回し試作fbe8305 (092238/092244) も主解2改善/0悪化/16同等で、根のみと全18最終値同じ。timtab1はnode738で35局所LP/.0067秒を使い近傍optimalだが改善なし。pg5_34はnode52で1局所LP/.094秒が4096pivotのiteration_limit、seedだけを保持し改善なし。追加LP/時間だけ増えたため後回しを棄却し、根のみへ戻す。ログと回帰履歴は保持。
- 次の対照は根の近傍を全体10%最大.5秒へ増やす単一変更。初期は全体5%最大.25秒でgmu両方が時間切れだったため。1024node/4096pivot制限と主解専用/再帰禁止は同じ。既定無効のまま18開発問で比較し、悪化も含めて選ぶ。まだ採用しない。

- .5秒予算の編集時に汎用文字列置換がdiving側へ当たったが、直後の差分/行確認で検出し、LP divingは従来の5%/.25秒へ復元、RINSだけ10%/.5秒へ修正。誤編集状態でベンチマークは起動していない。rins-larger-budget-tests.logを中間ログとして保持し、修正後118テストとrelease buildを別ログで完走した。
- 後回し版の最終監査も28解と全134cutが双方/元MPS Fractionで全合格。性能上は追加益がないため棄却理由は不変。

- .5秒試作c6b4657 (092558/092559) は3主解改善/0悪化/15同等。gmu50 -2532435.2987→-2574311.3903、gmu40 -1860472.4551→-2367540.5084、bppc4-08 72→69。初期.25秒版よりgmu50とbppが改善、悪化なし。根のみ/10%最大.5秒を既定化候補とする。118回帰と独立監査を確認し、既定化後の最終合成比較と4solver短期比較を残作業とする。
- 60秒対照を固定18開発問で開始。c6b4657のdetached worktreeへ同一バイナリとGMP/公式checkerをコピーしhashを保存 (.venv/raw MPSだけ不変原本へsymlink)。CPU4/5/6/7の4群、各1thread/16GiB、同じ問題を同じCPUでRINS off/on。通常開発CPU0/1やnativebaseline CPU8..15と重ねない。immutable binaryなので本体を変更しても長期対照は不変。artifacts/rins-60s-frozen.jsonとgroup1..4-launch.log、実結果はworktree内artifacts/smoke-*。初回freezeはcwdをsnapshotにしたまま原rootを相対取得し失敗したため、原rootを絶対指定してコピーし直した。最初の起動失敗ログも保持。先行60秒runは起動されておらず、正しく凍結した4群だけが進行中。

- .5秒版の独立監査は28返却解GMP絶対/公式双方全合格、全134cut Fraction有効。既定化後の試験設定挿入でtests/strong.rsの戻り型Limitsを関数本体と誤認し構文エラーになった。修正し、rins-default-tests.logを失敗ログとして保持、rins-default-corrected-tests.logで118テストが全成功。release buildも完了。長期対照のc6固定バイナリには触れていない。

- 最終既定2b4c09dの118 Rust回帰と小1000/中規模200は失敗0。新規9問×4solver (093203/093204) は全36時間制限/証明0、32返却解GMP絶対31合格/既知HiGHS pk1不正1/未検証0、公式32合格。Rust8解と全51cutは全合格。
- 60秒固定対照c6 (092942/092950/092951/092952) が全4群exit0で完了。18問3主解改善/0悪化/15同等、下界全同じ。gmu50 -2532435.2987→-2574311.3903、gmu40 -1888142.565→-2367540.5084、bpp72→65。36run中markshare両設定の2runがoptimal、残34時間制限。全30返却解GMP絶対/公式全合格、全134cutもFraction有効。結果とメタデータ/保存点を主repoへ回収、原frozen worktreeも残す。
- markshare_4_0両設定は同じ1486619 main nodesでoptimal約1。off32.289559秒/on32.161783秒、RINSによる証明差はない。元MPS検査に合格、公式MIPLIB objective1と一致 (検証用JSONだけに保存、optimizerへ未入力)。同じCPU4/1thread/16GiB/60秒の3solver追加094028はHiGHS主解1/下界0、SCIP主解1/下界0、GLPK主解5/下界約0で全時間切れ。3解は双方検査合格。Rustだけがこの単一の開発例で60秒内optimal。全体優位は未達成。
- native baseline09:33時点71/720、旧primary最適28、errors7、coordinator生存。7件の内訳はGLPK cbs-cta不正1、旧buildingenergy検査600秒切れ2 (新監査で双方有効)、GLPK ex10 reader拒否1、GLPK buildingenergy/bab2/bab6の3660秒watchdog3。watchdogは全てexit-9でresult未生成、進捗ログも保存される。新GMP別監査56解55有効/既知不正1/未検証0。稼働snapshot/共有依存/CPU8..15は不変。
- 次の候補は整数目的値の安全な下界丸め。まず原MPSのcost/offset包含区間を追加し、整数係数等式で唯一の非整数項の係数±1ならその変数の暗黙整数性を導出する。原モデルの任意の行乗数からLagrangian下界を原境界で外向き計算し、その保証がある場合だけ整数latticeへ上げる。LPの近似boundを無条件ceilすると誤枝刈りになるので禁止。初版は有限元境界が厳密小整数・伝播無効の場合に絞り、Nodeの既存boundsを実際の分枝制約として用いる。unbounded列の負reduced costは微小でも証明を拒否、必要なら乗数の微小scaleを別候補として再検証。mainの既存floatboundと区別し、密LP/条件不明は追加丸めをしない。原MPS Fractionによる独立な目的整数性証明と任意乗数の下界fixtureを先に作り、全列挙で誤枝刈りを検出する。まだ未実装。別候補: feasibility pump (主解なし問題)、強分枝で試した子のwarm基底のみの再利用。

## 2026-11-15: 保証付き目的下界の試作開始
- PR18を6d2e6edへmerge、certified-objective-bounds branch。詳細設計はOBJECTIVE_BOUND_PLAN.md。目的係数/offsetの原十進包含区間をModel/MPSへ追加。浮動小数点で整数へ丸まった小数はpointと扱わず、costとoffsetのprovenanceを分ける。
- 原整数変数と厳密整数等式のunit係数から暗黙整数性をqueueで導くGrid、任意の原行乗数と原境界によるLagrangian下界を外向き計算するCertifierを実装。非有界方向の微小負reduced costを許容しない。曖昧なpoint積和の符号は既存exact累算で再検査。まだ主探索への丸めは未接続、既定動作は不変。
- Python Fractionで元十進MPS/任意f64乗数の厳密下界1000fixtureを生成し、Rustで証明を返した場合は必ずそれ以下であることに合格。125 Rust回帰が成功 (極小積のunderflow/厳密0、暗黙整数性chain、微小負reduced cost、min/maxのlattice境界を含む)。初期2失敗はfixtureの一行MPSに3組を書いた構文誤りと、3.next_upの厳密下界からstep2で5へ上げられるのに3を期待した試験誤り。修正前ログも保持。metadata試験の比較演算子<-構文誤りも保存し修正済み。
- 原MPS Fractionの目的整数性proof checkerを既存cover checkerへ追加。OBJNAME/offsetとunit等式chain/費用gcdを別計算する。初期OBJNAMEテストもMPS一行3組のfixtureミスを修正。さらにfake infeasible+不正objective grid+cut空+主解なしの実プロセス試験で、benchmark_caseがcut空なら検査を起動しない実際のガード抜けを検出。objective_gridがあれば必ず検査するよう修正し、smoke/auditもfieldなしのobjective proofに対応。稼働baseline snapshotは不変。まだmain solverはobjective_gridを出しておらず、このガードは新機能接続前に修正した。

- 探索へMIP_OBJECTIVE_ROUNDING=0/1 (既定無効) として接続。原有限境界がpoint小整数・伝播無効・revisedの場合だけ、原ModelからGrid proofを作り、cut追加後LP Modelの原row乗数/boxで追加boundを保証する。RINS/probe内部では無効。原dual witnessをrow index/向きで戻し、必要なら乗数1±2^-40の別候補を全再検証する。通常float boundは従来通りで、追加lattice boundだけがこの厳密な検査を必要とする。全LP証明を形式証明化したものではない。
- mainLPの近似値は、rootまたはceilでincumbentへ届く見込みのあるnodeの計算選別にだけ使う。既にprune可能/丸め幅が微小/改善見込みなしなら余分な原行計算を省き、実際のbound採用には必ず外向きLagrangianを使う。flag無効や適用不可、補助LPではrow mapを作らない。
- 130 Rust回帰、1000 Fraction bound fixture、160 min/max全列挙が成功。step2/offset、下側/範囲制約、負rhs自由変数、RANGES端点消失ケースでの乗数復元、propagation/不明境界の適用拒否を確認。Python目的proof5/cover7/stage2回帰も成功。既定動作の数値自体は不変。次はflag onの小1000/二値1000/中規模200と固定18問の原MPS proof/point検査付き対照。

- 最初の130-test候補5ae740e (102142/102149) の18問対照は主解すべて同等、30n下界2.0297169811→3のみ改善。短期全36時間制限、markshare60秒102211の双方optimal約1、off31.115413秒/1486619nodes、on33.847151秒/1607635nodesへ悪化。追加ceilの正しさと速度は別で、既定無効を維持。小1000/二値1000/中規模200は失敗0。監査は30解GMP絶対/公式全合格、全154cutと全目的整数性proofも有効。
- ただしbinkarのoff/on双方が旧6794.860025から7394.229988へ悪化し、下界6646.232386→6646.313679へ上がった。cut proof照合で旧17本に対し双方24本と確認。cut前処理.25秒内に従来未完了だった3回目のLPが速く完了し、追加7cutで根の潜りが失敗、RINSもseedなしとなることを特定。追加目的丸めの効果ではない。root cut回数の上限をMIP_COVER_ROUNDSとして明示し、同じバイナリの2/4round比較を固定18問で行う。LP証明/制約を弱めて誤答を採用せず、少ないcutで既存の主解機会を保てるか調べる。まだdefault4。

- cut上限4/2の同一バイナリ対照2512662 (103229/103230) はbinkarだけ7394.229988→6794.860025と改善、他17主解同等。binkar下界は6646.313679→6646.232386へ約.081弱くなる。cut24→17で潜り候補を回復し、他の実cut件数は同じ。原MPS独立監査中。主解回復を理由に共通root cut上限を既定2へ変更し、MIP_COVER_ROUNDS=1..16で明示指定可能とする。目的下界丸めは別の任意機能で既定無効のまま。
- 最終130 Rust回帰とrelease buildが成功。新CLI objective-proofは最適化せず元Modelの目的整数性だけをJSON出力する。全240に対してFractionで性質を照合するcheckerを追加。holdoutはLP/MIP性能を測らず、元データ/証明の正しさだけを調べる。次に最終小1000/中規模200、最終8問4solver5秒、全240行列照合と目的proof検査を行う。

### 目的下界の根限定と最初の正の増分（実験中）

全ノードでの丸めはmarkshareの探索を約8%遅くしたため既定採用しない。`MIP_OBJECTIVE_ROUNDING=1`を根限定、`all`を全ノードに分けた。整数目的と有限な原境界からbaseと最初の正の増分を証明し、独立Fraction検査器で検査する。目的係数に依存する一般的な性質で、問題名や既知解は使用しない。保証下界に反する許容誤差だけの候補を拒否し、整数分枝の箱の目的最大値が保証下界未満なら枝刈りする。微小な下限1e-16の整数変数を0に丸める回帰を追加した。

追加した最初のテストは制約向きが誤っていて目的最小0だったため失敗した。G行へ修正し、最大化側も追加。修正後のRust全テスト、目的証明Python6件、既存cut証明7件は成功。ログはobjective-domain-corrected-*。実装は既定offのままで性能比較をこれから実施する。

根限定source8395126の18問対照 (110437/110438) は全36run時間制限、18主解全同等、30n下界だけ2.0297169811→51。1000小/1000二値/200中規模全失敗0、Rust133件/Python目的6+cover7成功。全240構造証明とmarkshare60秒比較は実行中。既定化はこれらの完了/監査後。native baseline 11:04 UTCで90/720完了、旧primary最適37、error8、coordinator生存。新GMP再監査も別ディレクトリで開始した。

根限定のmarkshare60秒対照110527はoff31.210640秒/on30.417432秒、両方optimal約1、両方1,486,619主ノード/1,494,216LPで同一探索。時間差を速度改善とは主張しない。2解と両構造証明はGMP絶対/公式/Fraction全有効。18問の28解と134cut、36目的証明も全有効。baseline再監査は保存73解中72有効/1無効/未検証0、唯一無効は既知のcbs-cta GLPK。新errorはchromaticindex512-7 GLPKの外部watchdog3660秒、途中ログを保持する。

全240入力の新base/増分付き目的構造証明は失敗0で完了。大規模square41/47等はFraction再読込が長かったが全合格。途中でsolve既定だけを変更したa0c6ea4のreleaseへ更新したため、構造証明の実装一致hashと前後binary hashをobjective-domain-proof-implementation.jsonへ保存した。性能比較の途中にはbinaryを変更していない。新既定の133回帰も合格。別worktree warm-objective-sourceで次の費用変更warmを試作開始し、release binaryは共有せず、このPRの性能測定に影響させない。

最終既定release a0c6ea4のfresh8問5秒 (111318、CPU0/1thread/16GiB) は8時間制限、Rust7主解全GMP/公式有効、51cutと8構造証明全有効。目的下界51と主解品質を再現。native比較は103709を明示再利用。root-onlyを採用しPRへ進める。

### PR20候補: 費用変更時の主単体法warm

PR19はa4705cbへマージ。別worktree warm-objective-source、branch warm-objectiveで開発しmainへrebaseした。行列構造一致と費用変更を区別し、primal feasibleな基底だけ主単体法へ渡す。違反ならcoldへ戻す。snapshot費用は更新、元Arcは不変。Stats集約のフィールド不足による初回コンパイル失敗を保持し修正。Rust136回帰成功。1000変更のcold/dense照合、別の1100解のHiGHS/Fraction検査も失敗0。部品時間はwarm0.1071秒/cold0.1468秒/HiGHS0.1440秒、pivot37035/53057、warm成功1000。小さい部品測定に限り、MILP全体の高速化とはしない。詳細docs/WARM_OBJECTIVE.md。次は主解専用の根feasibility pump。

### 強分枝の混合候補と実行不能hint試作

PR21はマージせず閉じ、main PR20からstrong-objective branchへ。候補を分数度と元費用×分数距離の混合にできるようにし、実行不能hintは独立flagに分けた。初期既定はいずれもfalse。8候補/256pivot/.1秒は維持する。評価候補indexと選択indexも出力し、非零費用の少数変数が実際に候補へ入るかを確認可能にする。子LPのbound/主解/証明の不採用は維持。両子実行不能でもmain node_limit/bound不変のテストと、混合候補が上位分数度6を保つ単体検査、80二値min/max全列挙を4設定へ広げた。実験計画はdocs/STRONG_OBJECTIVE_PLAN.md。

強分枝d9e9c41の18問4設定 (113501/113503) は全72時間制限、18主解全同等、併用だけ30n下界51→51.7547169811。56主解GMP/公式有効、268cut/72構造証明全有効。小1000/二値1000/中規模200全失敗0、Rust138回帰と80二値×4設定全列挙成功。30n60秒113850もclassic51/併用51.7547、両方主解なし。主ノード486→330/LP549→390は同時間枠の異なる探索であり高速化の証拠とはしない。両方のcut/構造証明有効。両設定を既定候補へ変更し最終確認中。

既定8e95f9eのfresh8×5秒114252でも30n下界51.7547と7主解の品質を再現した。native3参照は103709の同条件測定を明示再利用。新既定138回帰も成功。全件native baselineは11:43 UTCに99/720完了、旧primary最適41/error8、生存。次のdive-fixation-source worktreeで、一括整数固定を省く反例(min/max)と全139回帰が成功。releaseはmainの測定完了まで変更せず、PR採用後のrebase/対照実験へ進む。

### 測定開始前の検査器確認

PR23の一括固定解除は139回帰/訂正2200合成が成功したが、18問でbpp69→67/binkar6794.86→7429.76と利益・悪化が混在しLP増加のため未マージで閉じた。初回1657点の未検証はworktree checker配置漏れで、全失敗と訂正結果をfailed branchに保存。

事前確認だけをvalidation-preflight branchへ切り出した。smoke/syntheticはchecker未配置なら開始前にexit、validator単独のvalid=Noneは維持、smoke metadataにchecker hashを追加。preflight3回帰と既存validator回帰が成功。通常solverコードは変更しない。次は元十進の厳密整数行と整数point境界だけを使うi128の境界伝播を検討する。既存の任意float伝播は引き続き既定off。

測定ガードの作業中、mainのRustソースは潜り試作を含まないのに、共有targetのcached build後の実行ファイルにdive_fix_integralが出ることを0秒診断で検出した。新しい測定は開始していない。stale-binary-result.jsonと0.04秒のbuildログを保存。build-infoを追加し、compile時source_directoryが測定rootと一致することを開始前に必須化した。実際のmain再ビルド後はroot一致を確認。fake binaryの異なるroot拒否/同一root受理を追加しpreflight5件成功。今後はworktree固有targetまたは切替時の再構築+origin照合を使う。

origin単独確認の0秒protocol smoke120053は、binaryの出自がmainでもcached libraryが別worktreeという混在を検出し、採用せず保存した（nodes0）。libraryのcompile時directoryも定数で公開し、binary/library両方の照合へ拡張。fake mixed library拒否を含む6回帰が成功。新しいrelease build後は両方mainを返す。以後はworktree固有targetを必須にし、共有outputを使わない。新しいprotocol試験でsourceとchecker metadataを再確認する。

source directoryだけでなく同一rootの旧revisionも拒否するため、build.rsのstdlibだけでcore Rust/Cargo/build.rs内容のFNV64を生成しbinary/libraryへ埋めた。Pythonは独立に同じ入力を読み計算して照合、従来のSHA256記録も維持する。8preflight回帰が成功し、snapshot createへも照合とbuild.rs保存を追加。ソルバーの最適化処理は変更しない。

最終0秒protocol smoke120729はbinary/libraryのsource directoryとFNV fingerprint、checker SHA256が一致、nodes0/time_limitを確認した。新規snapshot preflight-snapshot-20261115の作成だけを行い、build.rs保存とarchive source fingerprint一致を確認、coordinatorは起動していない。初回setupはtaskset32なのにcores0を指定して開始前にCPU不正で停止し、cores32へ修正した失敗ログも保存。既存baselineへは変更しない。

### 既存の変数間積和相殺の誤答

整数伝播の純粋domain検査は成功しexact-integer-propagation branchへ保存したが、統合前に既存の数値誤答を再現した。exact-transformation-sums branchをmain001cbc5eから作成。固定x=y=z=1、費用1e16/1/-1e16で旧optimal0、GMPは目的1/誤差1。同じ固定点の等式1e16*x+y-1e16*z=1では旧infeasible。さらにx=1e16,y=1,free zの等式で旧optimal点にGMP行違反1。before出力と原MPSをartifacts/exact-shift-diagnosisに保持。旧binaryのbuild-infoも保存、compiled fingerprint19a8a69a7a6e183fの旧mainを診断した。

68limb積和を目的/offset/行shift/LP certificateへ使い、個々の積のoverflowも打消し前に失わない。最も狭いf64上下界・残差を求め、dualは下へ丸める。RHS/幅の絶対誤差が1e-9を超えれば数値エラーへ。Model行チェックは前方誤差区間→厳密fallback。Dense/Revisedの回帰は成功、1200 Fraction積和oracleも成功。最終確認/性能測定はこれから。詳細docs/EXACT_TRANSFORMATION_PLAN.md。


## 2026-11-15 13:55 UTC：PR25・元入力精度と積和相殺

旧mainの誤objective0/誤infeasible/無効なpointを元MPS GMPで再現し、厳密f64積和と元十進数のprecision guardを追加。f64の0.1を元十進数0.1と同一視した未採用試作3f1cd1cbも反例とともに棄却記録。主探索の非点データinfeasibleは元区間の零目的証明を要求し、証明器内の有限box/乗数候補を検査する。証明不能節点は残してnumerical_error。

最終ソース922e7cd4：Rust145、Fraction1200fixture、合成2700、純粋根LP300、全240行列/格子が通過。桁落ち回帰22件は8有効点/14未解決/誤答0。最終18問+mark60の14点はGMP絶対/公式とも全合格、70cuts/19runsの格子も合格。保存済み旧18問とのprimalは3改善/4悪化/11同等で速度改善とは扱わない。markshare約1の証明は35.750s（旧30.417s、同じ1,486,619節点）で負担増。gen-ip002は1節点の未証明infeasibleで数値停止が残る。比較native8問の21点中HiGHS pk1のみ絶対行違反1.000000008e-6、公式は全点合格。

詳細 docs/EXACT_TRANSFORMATION_RESULTS.md と docs/EXACT_TRANSFORMATION_PLAN.md。全生ログ/失敗を保存。固定snapshotはartifacts/runs/exact-sums-922e7cd4-20261115（createのみ、coordinator実行なし）。次のPair accumulatorは別worktree/別の実targetで開始し、source/binaryの混在を避ける。計画 docs/EXACT_ACCUMULATOR_NEXT.md。その後checked i128整数伝播checkpointも継続評価する。全Rust本測定は11月19日頃までに開始する期限を維持。

## 2026-11-15 14:10 UTC：厳密Pair/FMAの評価

別worktree/実targetでPoint/Pair/Wideを実装。和だけの初期版は18問のprimal全同等、markshare35.640→36.131sで不採用。通常Point経路の余計な演算を省き、最下位積bitがsubnormal下限以上と保証できる場合だけFMAの厳密残差を利用。最終50fe377dはRust147、Fraction3600bit fixture、合成2700、根LP300、十進回帰22件を通過。18問のprimal全同等。固定37,993節点のgenは3回中央値3.634→3.127s、CPU交換でも3.632→3.143s。ただし数値エラーの未解決節点で止まる状態は変えない。markshare目的約1の同じ証明は35.737→35.293s、CPU交換35.777→35.248sと小改善。保存68点/286cutsは独立検査全合格。反復で同名引数を重複して出力先衝突した診断runも保存し、個別3回で再測定した。詳細docs/PAIR_ACCUMULATOR_RESULTS.md、全データartifacts/pair-accumulator-evaluation（移動hash台帳あり）。一般的な大幅MIP高速化とは主張しない。

## 2026-11-15 14:36 UTC：整数伝播棄却から測定出力の衝突修正を切り出す

整数伝播の全節点/根限定はPR27で未マージとして閉じた。Rust155/両方式それぞれ合成2700/十進回帰22/最終56点280cuts全有効だが、全節点はmad主解とmarkshare証明時間、根限定は30n/mcsched下界が悪化した。コード・全列挙・生ログはexact-integer-propagation branchとPR27に保持する。

同じ秒にsmokeを開始すると出力先が衝突するプロトコル不具合だけをunique-smoke-runsへcherry-pick。mkdtempで一意なディレクトリを作り、Rustコア/設定は変更しない。既存preflight8テスト合格。同秒の2並行0秒run(143630)が異なる保存先に成功し、両方探索0/time_limit。artifacts/unique-smoke-evaluationに保存。次は根で固定された列のLP省略を検討する。

## 2026-11-15 15:51 UTC：大整数出力と比較の保存経路を修正

Rustの内部整数と短いJSON十進値の違いを修正。149テスト/3600oracle/2700合成/300根LP/240入力metadata・構造検査/22変換を通過。巨大整数10解、固定18問等28解・140cutsは独立検査全有効。18問primal同一、markshare同じ探索量で35.31→35.62秒。高速化とは主張しない。

比較側もHiGHS/SCIP巨大整数とGLPK15桁raw解の精度損失を確認し、完全整数JSONとGLPK直接API sidecarに統一。native8問×3×5秒は24件完走、21主解中20がGMP絶対有効、全21が公式相対検査有効。HiGHS pk1の行違反1.000000008e-6は絶対基準超過としてそのまま不合格。保存経路修正で全誤差が消えるとはしない。

旧baseline-v1は159/720完了でsuperseded。159結果・snapshotと8中断ジョブのcgroup資源記録を保持し、coordinatorの同一性を確認して停止、全case cgroupの空を確認。中断を算法のタイムアウトに数えない。baseline-v2を同一CPU8..15/1thread/16GiB/3600秒で新規開始。新driver hash329ec481ab751fa1b9cb540b5a986fb41b2b0a299d2bb707960cfdbd66ceffd4。旧snapshotは変更していない。最終Rust全240測定はNov19 UTC中までに固定する。

次はroot-fixed-columns別worktreeで固定列省略のrelease検査と比較。152テストは通過しているが未採用。データと移動hashはartifacts/serialization-evaluation。

v2の最初のnohupバックグラウンド起動はcoordinator identity生成前に消え、開始成功とは確認できなかった。ログが空のままの試行も保持する。待機する実行sessionで再起動し、15:51:44 UTCにcoordinator_alive=true、全8case cgroupで実行を確認した。以後はidentityと進捗で監視する。

## 2026-11-15 15:57 UTC：根固定列をLPから省く

元十進境界の外向き区間でも同じ値と証明できる根固定変数のみLPから省く。子節点の固定は構造変更しない。整数singletonも扱い、元FX0.3など証明不十分な連続定数は省かない。既存の厳密RHS/目的シフトで復元し、全列削除や定数行の実行不能も両engineで検査する。

152Rust/2700合成/300根LP/22変換/10大整数が合格。固定18問のprimal/dual/status同一、28保存解140cuts/38証明は全件独立検査有効。30n20b8の7282列を省略、同CPU交互3回の50節点/50LP・下界51の内部中央値5.064→4.140秒。根のみ0.08817→0.07579秒。標準10秒で48→57節点、同じ下界で主解なし。markshareは対象列0、同じ証明量で35.642→35.683秒。限定的改善として既定有効にする。新既定でも152Rustと32数値回帰は合格、CLIで省略7282列を確認。

最初の10問引数で存在しない問題名を誤記し両側n5-3後に停止した記録も保存、正しい固定10問を再測定。詳細docs/FIXED_COLUMNS_RESULTS.md、artifacts/fixed-columns-evaluation。全件native v2は720件のうち8件完了、継続中。次は行間の二値競合を集約したclique cutを、原行から独立検査できる証明付きで実験する。

## 2026-11-15 16:21 UTC：内部LP作業領域の設定・観測と512MiB化

256MiBの推定guardでsorrell3/supportcase10がLP開始前に停止していたため、MIP_LP_WORKSPACE_MIBと結果のlp_workspace_bytesを追加。既定512MiB候補で固定18問のprimal/dual/status全同一。2大規模開発問は15秒で1150/900pivots、23/18refactors、time_limitまで進み、RSS264468/264192KiB。主解/下界は未取得なので解けた数の改善とはしない。外部16GiB/1threadは同一。152Rust/32数値回帰/CLI6設定/26保存解/134cuts・40証明の検査合格。採用候補としてPRへまとめる。

clique-cuts（PR31）とrepair-breakout（PR32）は未採用で閉じた。コード・全ログは各branchに保存。次はLP前の箱候補で、LPが時間/メモリ制限になる場合にも元問題に実行可能な主解を残せるよう試す。native v2はPID556757で継続中、snapshot/依存は変更していない。

## 2026-11-15 16:46 UTC：LP前の箱主解を採用候補へ

box-primal（最終コアddbc8ea4）はLP前の箱端点+実行可能端点降下。gmu-35-50の10秒primalが-402939.9877→-2577545.7294、同じ下界でgap約1.18425%。3反復とCPU交換で再現。sorrell3はLP根未完了でも-14を取得。他17固定開発問のprimal/dual/status同一。native同条件比較はHS/SCIPにgmuで届かず、GLPKより良い。sorrellはHS-15/SCIP-11/GLPK-1。全体優位とはしない。

157Rust/200小例（184改善、188点が列挙最適値だがnode_limit）/300根LP/32数値回帰成功、主実験2700合成成功。46保存解/134cuts・50証明の全独立検査成功。許容誤差内の主解と根の実行不能が矛盾するとnumerical_errorで枝を保持し、誤ったoptimalを避ける。既定有効、機能別単体比較では新機能を明示無効にして条件を固定。詳細docs/BOX_PRIMAL_RESULTS.md、全生データartifacts/box-evaluation。

create-only snapshotはexperiments/box-primal-snapshot-20261115.jsonを参照。全件Rustは未開始。native baseline-v2はPID556757/session89978、12/720完了で継続中。次は純粋な二値競合を構造認識し、彩色上界付きのRust組合せ探索を検討する。mainのLP/主解を維持し、問題名分岐や既知解は使わない。

## 2026-11-15 17:24 UTC：純二値競合の構造専用探索を採用候補へ

binary-conflict-engineは全モデルを検査してから、同一整数目的の0/1・高々1制約だけをbitsetクリークへ変換する。負係数下側行の同値反転も対応。非対応はLPへ戻す。名前・既知解へ依存しない。sorrell3の15秒が旧primal-14/dualなし→-16/-28。最終3回とも再現。根彩色36の独立証明と、82～83根分岐完了後の内部探索上界28を区別する。公式既知値-16には一致するがstatus time_limitで未証明。固定18問のprimal/dual/status全同一。native同条件はHS-15/-20、SCIP-11/-225、GLPK-1/-512。

最終161Rust、300小グラフ×6の1800探索、CLI200×6=1200（全列挙/GMP/彩色、偽証明3600拒否）、2700合成、300根LP、32数値回帰成功。41保存解/134cuts・51証明検査も全有効。初回未認識と既定変更による機能別テストの干渉も保存。全データartifacts/binary-graph-evaluation、詳細docs/BINARY_CONFLICT_RESULTS.md。

snapshot7831746bはcreate-only、全Rust評価未開始。native v2はPID556757/session89978で継続、21/720完了・strict optimal15・errors0（17:10時点）。次はperf547sampleで約32%を占めたcolor内時計確認を間引き、固定節点・元問題解検査で比較する。open instanceへの追加探索も通常Benchmarkとは別枠で検討する。

## 2026-11-15 17:40 UTC：彩色中の時計確認を256選択ごとへ

graph-deadline-polling（コア3d18c03a）はcolorのInstant::nowを間引き、節点/分岐での確認は維持。同CPU交互3回・200万節点の内部中央値12.20450→5.99163秒（約51%短縮）。status/primal-16/dual-30/全変数/根彩色/順列/色分け回数/完了分岐数が同一。161Rust、1200小例、偽証明3600拒否、32数値回帰、小期限0/.2/.5/1秒、12保存点の全独立検査成功。profile時計比率約32→1.65%。詳細docs/GRAPH_DEADLINE_RESULTS.md、全生データartifacts/graph-polling-evaluation。

sorrell7をBenchmark外explorationへ追加（hash記録済み）。既知値はコード/停止条件に渡さず15秒で旧404万→新795万節点、両方primal-168/dual-449。掲載値-198未達。通常240問の集計には混ぜない。今後は汎用の主解改良も試す。

native v2は22/720、strict optimal15、errors1（17:33時点）。020-glpk/cbs-ctaが元問題検査不合格、行違反約1.449。直接APIの完全binary64十進点でも絶対検査不合格なので保存精度だけの問題ではない。補助完全十進点の公式checkerは32桁assertで判定不能、通常JSON点の公式checkerはConstraints0。全raw保持、比較設定/snapshotは変更しない。coordinator PID556757/session89978継続。全Rust最終版はNov19中に固定する計画。

## 2026-11-15 18:05 UTC: 競合構造上の局所探索

b45eeaf8を採用候補とした。sorrell7の15秒主解-168→-187を追加3回で再現、下界-449は不変。native同条件はHS-186/-236、SCIP-152/-670.6667、GLPK-21/-1024で、下界ではHSが強い。全保存点の元MPS検査成功。固定18問はprimal/dual/status一致、162Rust・最終1200CLI全列挙・32数値回帰成功。詳細はGRAPH_LOCAL_SEARCH_RESULTS.md。全件native継続、Rust snapshotはcreate-only。次は根の良い彩色を汎用Rust実装へ移し、既存の根prefix上界証明との整合を確かめる。

## 2026-11-15 18:12 UTC: 根彩色を改善

根だけ共通近傍/飽和次数の完成彩色を試し、元の完成彩色と最良を選ぶ。sorrell7の根色数449→379、15秒と60秒のdual-449→-379を主解-187のまま再現、追加15秒3回も一致した。構築約0.039秒。sorrell3の根36→31だが15秒最終-16/-24は同じ。固定18問の主解/下界/status一致、162Rust/7200内部全列挙/1200CLI/32数値回帰成功、42保存点GMP絶対/公式検査有効。初期のバイナリ更新によるsmoke中断とCPU重複は無効試行として保存し、専用CPU5/6で正式再測定した。詳細GRAPH_ROOT_COLORING_RESULTS.md。次はGRAPH_NEXT_HYPOTHESES.mdの彩色修復または局所探索幅を調べる。

## 2026-11-15 18:23 UTC: 元の根順と複数彩色prefix

PR39の局所探索予算増加は閉じて未マージ。次の59dac347試作は元の根順を維持し、完成した全彩色をprefixへ制限した色数の最小値を上界とする。sorrell3最適性証明の3回中央値20.483→18.199秒、6777969→6042153節点。sorrell7の15/60秒は-187/-379を維持。固定18問primal/dual/status一致、162Rust・14400内部全列挙・1200CLI/7200偽証明拒否・32数値回帰、43保存点GMP絶対/公式検査成功。既定有効へ変更し最終検査中。native60秒sorrell3はHS-16/-20、SCIP-11/-76.5、GLPK-1/-255で全時間制限。一問の結果として保持し、全体優位とは扱わない。

## 2026-11-15 18:30 UTC: 局所探索の再構築を多様化

2526036f試作で最良集合の一部を外し、固定seedで追加順を再構成した。予算0.25秒のままsorrell7の15/60秒主解-187→-190、下界-379を維持。追加15秒3回で再現。sorrell3は最適性証明6042153節点と18.37/18.39秒で変化なし。固定18問主解/下界/status一致、162Rust・28800内部全列挙・1200CLI/7200破損証明拒否・32数値回帰、37保存点GMP絶対/公式検査成功。既定有効へ変更して最終検査中。次は全件評価のソルバー間bound整合性診断を整備し、局所探索の改善余地と汎用LP/MIP機能も継続する。

## 2026-11-15 18:36 UTC: 全件結果のbound整合性診断

PR41で局所探索多様化を採用し、sorrell7主解-190/-379を再現した。次にcheck_bound_consistency.pyを追加。元入力/結果SHAとMPS senseを確認し、GMP絶対検査済みの主解と各報告dualをDecimal/Fractionで照合する。要確認は誤差0の実行可能性・目的値等の再検査が必要で、dual証明や自動的な誤答認定とは扱わない。13CLI合成検証成功、途中native v2の13入力/30boundで要確認0・警告0。稼働中snapshotは読み取りのみ。次は新しい多様化の局所探索に対する適応的予算を、前の予算増加不採用と区別して検証する。

## 2026-11-15 18:42 UTC: 多様化した局所探索の適応停止

最大2秒/128nを許し、16n改善しないと止める案でsorrell7の15/60秒主解-190→-191、下界-379を維持、追加15秒3回でも再現。sorrell3は約0.148秒で停滞終了し証明18.439/18.489秒、同じ6042153節点。固定18問主解/下界/status一致、162Rust・57600内部全列挙・1200CLI/7200破損証明拒否・32数値回帰、37保存点GMP絶対/公式検査成功。既定有効化して最終検査中。次はLP_EXACT_POINT_HYPOTHESIS.mdの誤差計算の同値な簡約を検証する。native v2は31/720・strict optimal16・error1で継続中。

## 2026-11-15 18:50 UTC: 整数値shiftの誤差計算を簡約

be70fe46で既にpoint_error=0と確認したshiftだけ、dxを掛けるゼロ項とx=0での係数誤差計算を省略した。gen-ip002の固定37993節点、同じCPU7/1thread/16GiBで3回中央値3.1605→2.9980秒（約5.14%）。時間項を除く全返却情報が完全一致し、既存numerical_errorも維持した。163Rust、5000行bit一致、2700合成MILP/300根LP/32数値回帰、固定18問主解/下界/status一致、33保存点GMP絶対/公式検査が成功。次は残るRow活動誤差/warm_impl/Basis前後退処理をprofileから改善する。native v2は31/720完了で不変snapshotのまま継続。

## 2026-11-15 19:07 UTC: 基底に属する双対乗数キャッシュ

31ef9ff5で同じB^-T c_Bを不変Arcへ保存し、入力bit一致時に再利用。ピボット/再分解で破棄し、親子間で不変共有。入力iterator比較でヒット時Vec確保も省いた。同じCPU7/1thread/16GiBの3回中央値gen-ip002 2.98799→2.94067秒（1.58%）、Mark100000節点2.44036→2.39382秒（1.91%）。cache統計と時間以外の全返却情報が一致。gen numerical_errorも維持。164Rust/2700合成/300根LP/32数値回帰、固定18問主解/下界/status一致、正式38点GMP絶対/公式検査成功。初期試作38点とコンパイル失敗も保持。native v2は39/720完了・strict optimal18・error1で不変のまま継続。次はRHS依存しない価格計算・warm構造や前後退処理を更にprofileし、広い開発問題で検証する。

## 2026-11-15 19:22 UTC: 目的値の相対判定を明示し厳密再計算値で照合

大きな係数の3binary例で報告目的値とGMP厳密再計算値が1違うことを確認。最初は絶対検査の問題を疑ったが、既存方針は制約/境界/整数性が絶対1e-6、目的が相対1e-8で、文書とGMP実装どおりだった。主解全1・最適性の誤りではないため評価基準は変えず、objective_evaluationへ基準と絶対誤差を明示し、primal_bound_exact/診断reference_pointsへGMPの厳密値を保存する。16集計/2GMP/8preflight/32数値回帰成功。native v2の15入力/39boundで要確認0。snapshot不変、再最適化不要。詳細OBJECTIVE_REPORTING.md。次は広いMILP/LP性能と数値失敗の改善を継続する。

## 2026-11-15 19:42 UTC: 元問題に証明できないLP不能を保留

未証明領域の継承下界を保持して別ノードを探索する試作5f1e824fで、gen-ip002の10秒主解-4729.769→-4752.503（追加3回も再現）、60秒-4758.674へ改善した。dual-4840.542は不変、保留23/141領域が残りstatus numerical_errorを維持、実際の終了理由time_limitも別保存。全nativeの10/60秒主解・下界はなお本実装より強い。166Rust/2700合成/300根LP/32数値回帰、固定18問はgenだけ主解改善、保存39点GMP/公式検査成功。既定有効へ変更して最終テスト通過。次は数値失敗後により良い下界の領域へ探索を移す汎用戦略とLP安定性を検証する。
最終の最大化メタモルフィック検査でも、50000節点で元と同じ点・処理量・保留7件、逆符号のprimal/dual/保留boundを確認した。保留boundの出力は内部最小化座標から元の目的方向へ戻す。GMP/公式検査も成功し、保存41点となる。

## 2026-11-15 20:28 UTC: 同時乗数調整による不能証明の補完

節点有限箱の追加推論は最初のgen-ip002失敗で17個の上限を補えず、node-proof-boxブランチへ不採用として保存。複数Farkas乗数の同時±調整を元の区間証明器に渡すと最初の失敗は7候補目で保証できた。上位8乗数/最大512候補と期限確認に制限。gen10秒の未証明23→9、60秒140→53、固定100000節点13→5、主解/dualは全て不変でnumerical_errorも残る。固定18問も主解/dual/status一致、2700合成/300根LP/32数値回帰成功。改善は証明できる範囲であり解品質の改善ではない。次は残る失敗dumpに対する冷LP再解と証明候補の安定性を調べる。native全件v2は49/720完了で継続。

## 2026-11-15 20:36 UTC: 元制約の非負結合で証明専用境界を補う

保存20件を冷LPで解き直しても証明改善は1件だけだったので未統合。制約の非負結合から根問題の有限箱を区間演算で導く案は全20件を証明し、実行gen10/60秒でも失敗9/54→0、numerical_error→time_limitへ変化した。主解/dualは不変。固定18問の主解/dual一致、gen以外status不変、2700合成/300根LP/32数値成功、保存30点GMP/公式・134切断検査成功。既定有効版と固定節点比較を最終確認している。次はbinary feasibility pump等の汎用主解手法を調べる。native全件v2は52/720完了、strict optimal21/error1で継続中。

## 2026-11-15 20:48 UTC: 初回主解に限定したbinary feasibility pump

無条件pumpはgmu-35-50を悪化させたため不採用。初回主解なしに限定すると固定18問の主解/dual/statusは全て直前版と一致。binkar10_1の初回時刻3回中央値0.39120→0.27456秒、最終点は同じ。追加開発6問中blp-ic98で10秒主解なし→18526.1675を発見、追加3回同じ点。nativeの10秒主解/下界は両問題ともなお強い。2700合成/300根LP/32数値、初回限定で追加1500合成、173Rustテスト成功。候補7412.420071を別保存しGMP/公式検査成功。全保存点の再検査中。次は整数固定後の元目的LPによる連続部分polishingを調べる。

native v2は57/720・strict optimal24・error2。新しい044-glpk/ex10はGLPK入力parserが元MPS 69612行でmissing row type in field 1、driver exit3、worker exit1となり結果ファイルなし。精度捕捉とは別の入力読込失敗として原ログを保持し、測定snapshotや720件計画は変更しない。

## 2026-11-15 21:09 UTC: 整数乗数候補でLP不能証明を補完

通常証明が失敗したときだけRust連分数から小分母比/共通整数乗数を提案し、既存の外向きcontradictionで必ず再証明する。glass4は92節点/主解なし→163節点/主解3466706233.3333354、追加3回同じ点と40補完を再現。後続raw LP数値失敗は残る。app1-1は59節点失敗で不変。全native10秒の点/下界はなお優れ、HS/SCIPはapp1-1を最適証明。177Rust/2700合成/300根LP/32数値成功、固定18問主解/dual/status一致、37保存点GMP/公式/140切断検査成功。全件Rust snapshotはcreate-only、native v2は63/720・strict optimal25・error3で継続。次はfresh LU後のratio失敗を残差から診断し、閾値を緩めず基底解の精度を改善する。

## 2026-11-15 22:02 UTC: 重み付き頂点被覆の主解探索

汎用binary conflict graphで独自Rustの重み付き被覆局所探索を導入し、sorrell7の15秒主解-191→-193、60秒-197、下界-379は不変。最終版追加15秒3回/60秒2回も再現。Benchmark v2外で240問へ加算しない。公開最良-198超え/最適性証明は未達成。sorrell3は同じ6042153節点、証明時間中央値18.44847→18.91649秒（2.5%遅い）を代償として記録。固定18問は全出力主指標一致、180Rust/1200CLI/7200破損拒否/2700合成/300根LP/32数値/8preflight成功、64保存点GMP/公式/134切断検査有効。新しい子モジュールをsnapshotへ再帰コピーする修正も確認。次は分数クリーク被覆の上界を独自Rust/独立証明で調べる。nativeの候補重みはメイン最適化へ渡さない。

### 2026-11-15 22:19 UTC 分数被覆上界の試作

`graph-fractional-cover`で独自Rustのクリーク回数ヒューリスティックと整数証明を実装中。既定off。初版sorrell7 15秒でprimal -193維持、dual -379→-306、sorrell3は-16/-24で未改善。4点の元MPS検査成功、1200小規模CLI/7410破損拒否成功、Rust全体テスト成功。構成順序を変える次版を測定中。計画/証明/初版結果は`FRACTIONAL_COVER_HYPOTHESIS.md`、ログは`artifacts/fractional-cover-*`。比較用native LPの重みは最適化に使わない。native全体比較は77/720完了・32件独立検査済optimal・5件error、凍結設定を変更せず継続。

### 2026-11-15 22:28 UTC 分数被覆の最終設定

source core `3125227a`、既定有効。sorrell7 15秒193/242、60秒197/241、追加15秒1回/60秒2回も再現。主解は直前版と同じで上界を379から強化、公開最良198超え/最適性証明は未達成、Benchmark v2外の探索例。sorrell3の同一CPU3組は6042153節点不変、中央値18.85908→19.37730秒（+2.75%）と悪化。固定18問status/primal/dual不変。181Rust、最終1200CLI/7410破損拒否成功。全240問snapshotはcreate-onlyで再帰FP検証済、receipt `experiments/fractional-cover-snapshot-20261115.json`。native全体は78/720、32独立optimal、5errorで設定を変えず継続。

次の仮説は短時間でも全頂点に届く安価なmaximalクリーク生成。sorrell3の比較専用native被覆LPは別候補20000個で約23.004、Rust候補の偏りが疑われる。native重みは一切使わず、現在版のPR確定後に別branchで先頭bitによるgreedy maximal化等を比較する。詳細はFRACTIONAL_COVER_HYPOTHESIS/RESULTS。保存解の一括独立再検査・原データarchiveを作成中。

### 2026-11-15 23:02 UTC 二行集約切断を検証

`aggregate-rounding-cuts`。原行の正整数結合+既存床除算、証明を元MPSで再合成する独自Rust切断を実装。10秒固定18問はtimtab1の下界28694→52566のみ改善、他17問status/primal/dual一致、60秒も改善維持。主解は両方なしでnative10秒下界374243〜652312にはなお大幅に劣る。既定trueへ変更、非負整数対象がないモデルは新準備をskip。183Rust/2700合成+最終500/400根LP/32数値/8独立checker/8preflight成功。初期保存28点GMP/公式有効、38記録175cut有効。toyの許容差0テスト設定、根LP比較で新cutを切り忘れた2ケース、Dense単位変換既存50失敗はログ保持。Dense50はPR56凍結対照と全指標一致で回帰なし。次は開発188問の根LP診断を広げ、holdout52は触れない。

原native全体80/720、32独立optimal、6error。追加HiGHS/chromaticindex1024-7は3660.18679秒watchdog/SIGKILL/resultなし、OOMなし。missing resultを解けた/正常timeoutに付替えず保持する。

### 2026-11-15 23:05 UTC 集約切断の採用前監査

最終source core876099b61/FP8262e2efdce05367、create-only全240問snapshotを再帰FP照合済。通常34点GMP/公式有効、46記録222cut独立検査有効。別に数値32fixtureは専用GMP/Fractionで有効。通常経路で追加監査した6数値記録は4reader拒否・2公式checkerのobjective行RHS parser abortで未検証となったため、公式合格と混ぜず失敗試行を保存。製品やchecker許容差は変えない。最終unit183、最終500合成、孤立400根LP、32数値、checker8/preflight8、既定trueのtimtab10秒52566再現。PRへ全原データと全失敗を掲載する。

### 2026-11-15 23:21 UTC 集約行512と開発188問

集約候補128→512でtimtab1の10/60秒下界52566→82576、主解は両方なし。fhnw10/60秒はなし/0不変、残り16固定開発問は主指標不変。183Rust成功。証明はPR58と同一で、試作native LP値を自作Rustで再現後に採用判断。`aggregate-row-diversification`に記録。比較script初回SyntaxErrorの2ログと旧ソースを保存。

凍結PR58で開発188問だけの根10秒/1節点診断を完了。77根点/91time_limit/11resource_limit/9numerical_error、driver/watchdog異常なし。77点は元MPS Fractionの連続LP制約/境界1e-6/目的相対1e-8検査すべて有効。MILP77問を解いたとは数えず、他111未検証と区別する。内部作業領域512MiB→2GiBの11追加診断は10件LP開始/根点なし、1件resource継続。既定メモリ値は変更しない。次は根数値エラー9件の原因と安全な再試行を調べる。holdout52は触れず、native全体は進行中。

### 2026-11-15 23:47 UTC 根基底残差補正に新しい信号

PR54の比率検定時だけの補正を9新根失敗で再評価。germanrrは17.2406秒/7923pivot/補正11成功で根LP完了、元MPS Fraction連続LP検査有効、native LP値と一致。main60秒は従来数値停止から119節点/下界45980135.4164/主解なしへ進行。glass4/app1-1は現PR59と全主指標同じ。固定18問も全主指標不変。185Rust/2700合成/400根LP/32数値成功。新native60秒germanrrはHS主解47761749.7868/下界46387816.7460、SCIPなし/46782236.6511、GLPKなし/46096013.6615で自作より強い。監査後に採用判断。旧patch競合は対象関数だけ抽出して解消、旧後段補正/候補受理コードは取り込まない。

列順逆転は9意味同値入力で根点0、単独採用しない。内部作業領域2GiB/60秒も11件で根点0（10時間切れ/1resource）なのでメモリ既定を維持。次は真の基底境界違反と目的0の双対法による回復を調べる。

### 2026-11-16 00:14 UTC 目的0の基底回復に新しい根点

残差補正で直らない基底に、目的0で既存dualを適用。最大4回/既存予算、普通dualの500退化停止は不変、補助だけは目的0でbeta常時0のため総pivot/time capを使う。600独立行を600pivotで回復する例など188単体成功。neos-860300根は回復1pivot後14.962sでLP完了、元MPS Fraction制約/境界1e-6以内、LP値1666.0685857415はnative LPと一致。main60は従来数値停止から128節点/下界1667/主解なしへ。HS/SCIP native60は3201最適、GLPK4212/2122でなお強い。

初回実行時bool共有版はbinkarで7069.30→7093.73へ悪化し、呼出0回でも同CPU3組再現。const genericで通常/補助を分離すると3組とも7069.30/同下界へ戻り、固定18問主指標も全一致。原因詳細は断定せず全コード/ログを保持。最終監査/合成再検査後に採用判断。次は追加切断行のWarmStart拡張と大きい根での切断生成（WARM_ROW_EXTENSION_HYPOTHESIS）を調べる。

### 2026-11-16 00:19 UTC 目的0回復の最終検査

const generic分離版は188単体、最終2700合成/400根LP/32数値/checker8/preflight8成功。45保存整数点GMP/公式、61記録466cut独立導出有効。neos-860300 main60は数値停止から128節点/下界1667/主解なしへ、最終根14.962s/1回復pivotで原MPS LP検査有効。native60 HS/SCIPは3201最適、GLPK4212/2122で自作はなお弱い。固定18問主指標全一致。binkar初版の再現した悪化はconst分離で3対照組とも解消、旧版/ログを保存。

source8b912389e/FPc9d1617c4877d996、全240問create-only snapshot照合済。次は追加行WarmStartと根cutの予算分離。native全体104/720、検証済最適45、error7で凍結条件を継続。

### 2026-11-16 06:11 UTC PR62–71の採用と現在の基準

PR62で追加cut行の基底再利用、PR63で最初の根LPとcut予算分離、PR64で証明付きclique、PR65でcover4巡を採用した。PR66の静的正規化価格付けは悪化があり未マージ。PR67で所有Colの余分なcloneを除き、PR68でfaer一列Mat RHSへ変更し、Colのstride0による行ごとのコピーを回避した。PR69は元の根cut終了後のMIR試行を、相対LP改善または整数点取得の場合だけ保持する方式で採用。local probe/RINSにはMIRを渡さない。詳細は各RESULTS文書とPR原データを参照する。

PR70は零目的LPを従来法で1秒先行し、長引く場合だけPhase IをL1重み付き価格付けで再開始する。krkaは従来60秒でLP点なしから約3.4秒で有効LP点を得たが、整数解の改善ではない。即時正規化は生成LPでも遅くなり棄却した。PR71は最初の根LP再計算がprobe全予算以上になる場合だけ、根の有効cutモデルと不変基底を局所主解探索へ渡す。nu25の10秒54875→54375、60秒54810→54375を確認。native HiGHS/SCIP53905最適・GLPK53925にはまだ及ばない。一律再利用でのeil33悪化は条件付き版で回避。

現在のmainは8f70b1e45、Rust FP839469b484e9f3e4。PR71の188組は2主解改善/0悪化、107保存整数点GMP/公式有効、376証明Fraction有効。h80の改善は強分枝offで差が消え、非発動loueのdual差も直接効果とは断定しない。210 Rust/2700合成が通過。最後のcfg(test)整形の前後でrelease .text SHAは完全一致した。全240/3600秒snapshotはprobe-root-warm-formatted-final-20261116を作成しただけで未実行。独立LP回帰は現在300設定・500 engine出力（通常/行目的scale/変数単位）と数え、18有効点+14精度未解決からなる数値回帰32件と混同しない。

混合モデルのbinary pump制限解除、上限1→5秒、射影反復数拡大はいずれも主解改善がなく未採用。枝名mixed-binary-pump/pump-projection-budget/pump-projection-work-budgetに証拠を保存。pg指定はsplit検査でモデル読込前に拒否され、holdoutで実行していない。

### 2026-11-16 06:11 UTC 双対回復の失敗時再試行を評価中

一律に零目的dualの同率候補を大きいpivot優先へ変える案はranの数値停止を解消し10秒整数解4459を得た。元MPSのGMP絶対誤差/公式とも有効。native同CPU41/10秒はHS3806、SCIP3987、GLPK4538で、自作はGLPKより主解だけ良く、他指標ではなお弱い。しかしneos860300の60秒は3966/2031→なし/2002へ悪化し、根LP15秒→58秒となったため一律版を棄却。188の10秒だけではこの悪化を見つけられなかった。LU再分解だけの案と回復上限4→8もniddaを直せず棄却。

現在は従来dual回復がNumericalErrorになった場合だけ保存状態から安定pivotで再試行する。失敗側の仕事を総予算に残す。PR71へrebaseしたcore968487c72/FP3365eb6d5aedf8c8を固定し、mainFP839と188開発問題・10秒・1thread・16GiB ASで比較中。統合前の209 Rust/2700合成/500 LP出力/32数値と、統合後210 Rustが通過。統合後2700合成と全保存点検査を実行中。統合後neos60は3966/2031、germanrr60はなし/45980135.4164を維持。採用未定。

最終全240の比較、native全体優位、世界記録は未達成。期限は11月22日14:59 UTC。最終Rust全件実行を11月19日UTC目安まで遅らせず、holdout52を調整に使わない。全失敗もraw statusで残す。

### 2026-11-16 08:03 UTC PR72採用後と進捗に基づく復元

PR72をmergeしmain45257f737/FP3365へ更新した。失敗したdual回復だけ安定pivotで再試行し、統合188対はranの主解1改善/0悪化、107点/376証明有効、neos860300/germanrr60秒維持。詳細はDUAL_RESTORATION_RESULTS。全件snapshotはcreate-only。

次の復元案FP8b1969fa725c8ce2は、Phase I目的値が明確に進んだ場合だけ連続復元4回の基点を進め、総pivot/flip/deadlineは返さない。nidda根LPを79pivot数値失敗→431pivot・有効LPへ回復し、MIP10秒で有効主解14054.95を得たが70nodeで数値停止する。188対は主解1改善/0悪化/187同値、dual1悪化loue22966→22953、376証明/109整数点有効。211 Rust/2700合成/500 LP出力/数値32（18有効点+14精度未解決）を通過。非発動loueの根cut予算は上限0.25秒で、strong off/60秒でもcut本数差が残る。状態管理を通常経路から分離する対照版でも差を解消しなかった。小幅のdual悪化も明記して元案をPR準備中。

双対逆積/失敗時primal方向の残差補正、数値停止後PhaseI再始動、PhaseI停滞時box矛盾候補、数値停止後の強い平衡化再試行はいずれも追加の有効点/解品質改善がなく未採用。専用branch/原出力/検査/凍結sourceを保存する。初版復元ELFはcargo testに上書きされ、source FPを再構成一致させたがrebuild ELFのbyte SHAは元と異なった。この初期診断の制限を保持する。採用候補FP8bと全件snapshotのELF/source/hashは測定前に凍結済み。

3行集約は別branchで既定false。10秒188対は主解0改善/1悪化（timtab1）・dual3改善、376証明/108点有効。timtab60秒では1089832→1072270と改善するため、全188対の60秒on/offをCPU3..7で継続中。さらに十進行の厳密整数倍MIRは別branchで明示flagのみ。元MPS Fractionを使った77点154分離出力で十進由来66cut/11問題の証明がすべて有効。213 Rust/10 Python検査通過、8問10秒trialの16出力/10点も有効。ただしbinkar10秒主解6888.55→6994.55が悪化しdual6646.31→6669.29は改善、60秒対照中。両案ともまだmainへ混ぜない。

native全240×3は凍結条件で継続し、08:02 UTCは209/720。新error055-glpk（validation split、gfd...）は3660秒PG watchdog・OOMなし・出力なし。holdoutのモデル内容は調べずログ/資源/実行状態だけを記録した。全失敗をrawのまま残す。最終Rust全240評価はまだ未開始。

### 2026-11-16 09:08 UTC PR73採用、3行集約の統合候補

PR73をmain c54dfda2d/FP8bへmerge。188対の主解1改善/dual1悪化を両方残し、全240snapshot restoration-progress-final-20261116は作成照合のみ、未実行。AI最新PRコメント累積654560522 tokens。

3行集約の旧基準188対60秒は主解1改善/0悪化・dual3改善、376証明/118点有効。main73統合・既定ON候補FP337114ffc2f9a6b7は214 Rust/2700合成/500純LP/数値32が通過。統合188対10秒は主解0改善/1悪化・dual3改善、376証明/110点有効。timtabは10秒悪化を残すが統合60秒1089832→1072270を再現し、neos/german/nu維持。主workspaceのnative同CPU60秒も再現したがHS/SCIP最適764772には及ばず、GLPKには主解だけ良い。詳細AGGREGATE_THREE_ROW_RESULTS。現在PR準備、十進MIRは混ぜない。

十進MIRは188対10秒主解0改善/1悪化・dual1改善、376証明/110点有効。binkar60秒も主解悪化。未採用としてdecimal-row-mir branchに4996 files/170316448 bytesをSHA台帳付き保存。疎LP内部workspace512→2048MiBは6例10/60秒で点なしのため未採用。制約追加と連続実行可能性のRust診断も4例で新規主解なし（supportcase12の全0点は既存box候補と同じと確認）、未採用。partial-phase1-pricingは212 Rust/最後block検査を通過し生成LP6例で約8%短いがgermanrr LP17.9秒→60秒点なしへ悪化し未採用。原出力と検査を各branchに保持。

native全件は09:03 UTC 224/720、厳密最適103、error15。新070-glpk k1mushroomは3660秒PG watchdog、OOMなし・出力なし。凍結条件を変えずraw errorのまま報告する。最終Rust全240評価は未開始、期限11月22日14:59 UTC。

## 2026-11-16 09:57 UTC 未採用LP実験の再開可能な保存

PR74をdf80ef0fdへマージ済み。既定FP337114ffc2f9a6b7、全240用aggregate-three-final-20261116は作成/ハッシュ照合のみ。最終自作全件は未開始。

内部LP workspace拡大、制約追加/連続実行可能性、partial Phase I pricing、停滞時の証明候補回復をartifacts/lp-numerical-negative-evaluationへ集約。2209ファイル97,389,550bytes、ELF4本はローカルに残しSHA/凍結ソースで参照する。元出力とのSHA relocation-ledgerを作成し、誤った設定名・CPU競合で破棄した測定・失敗ログも残した。詳細は対応HYPOTHESIS/NUMERICAL_CERTIFICATE_RECOVERY文書。

workspace512→2048MiBは6件でresource_limit→time_limitだが根LP点なし。制約追加で得たsupportcase12全0点は既存box候補と同じ。partial pricingは生成LP6例約8.3%短縮に対してgermanrr LP17.9→60秒点なしの悪化があり棄却。証明候補回復はnidda70→93ノード、新受理14証明を元MPSでFraction独立確認したが主解/下界不変。V1/V2の8出力の証明と4整数点は検査成功。全体速度/解品質の改善として数えない。

数値停止93枝は比較HiGHSでLP実行可能。自作改訂/密表は単独でも停止、既存float伝播後の改訂LP点は元MPSのFraction検査成功。ただし旧伝播を新しい小数係数の証明に使わず、係数/RHS/境界を区間で囲む別試作を進行。単独区間回復では139ノードまで進むが解品質不変、100小数行の有理格子全列挙で実行可能点の保存と期限切れ無操作を確認。採用せず次の診断へ。

nativeは09:53UTC 234/720、厳密最適108、エラー16。063-scip highschool1-aigioは3660.49秒watchdog終了、ログを今回保全。比較条件は変更しない。holdoutの入力を改善判断に使わない。

## 2026-11-16 10:36 UTC 偶奇等式の一意解証明

main PR75以降のparity-equality-solverで、私有±2整数補助を持つbinary等式をGF(2)で解き、full rankの場合だけ一意な元整数点を厳密復元する経路を作成。FP527633393c003557、MIP_PARITY_EQUALITIES=0/1、既定ON候補。元MPSのFraction＋独立streaming GF(2)消去を通常のstrict検査へ統合。正確な整数へ丸められただけの元係数/RHS/境界は受理しない。

188開発同ELF/各10秒は主解1改善/0悪化/187同値、dual1改善/0悪化/78同値/109比較不能。enlight_hardを主解なし/dual16.4583→独立証明付きoptimal37。376出力/111整数点すべて有効。swathは同品質でtime_limit→numerical_errorの状態差が10.001秒付近にあり、原出力を残す。別0/1対照では両方time_limit。60秒7問でもenlight以外の品質不変、16出力/13点の監査成功。

219 Rust（300全列挙含む）、CLI生成200（175最適点GMP/25不能列挙、90一意性証明/270改変拒否）、通常2700合成、LP300設定500出力、数値32（18点有効/14精度未解決）を通過。主workspaceでも再ビルド/同資源native10秒比較: Rust37最適0.000153秒、SCIP10.0.2は37最適0.003572秒、HiGHS1.16.0は主解なし/dual36、GLPK5.0は主解37/dual25。既知最適値の再現であり新記録ではない。主workspace binkar/niddaは従来品質、5点/23切断・格子証明も有効。最初のworktree GLPK起動失敗は原ログと再実行を保全。

artifacts/parity-equality-evaluationに5917一次ファイル183,165,108bytes、ELF6本ローカル参照/生成GLPKモデル3省略。heuristic-recovery-negative-evaluationには未採用の区間回復/完全ピボット/一般整数pump 2557一次ファイル47,211,813bytes、ELF6本ローカル参照。全relocation-ledgerを照合。全240用parity-equality-final-20261116はcreate-onlyで全hash照合、最終自作全件は未開始。

native凍結評価は10:26UTC 240/720、厳密最適108、エラー17。077-glpk map16715-04は3660.23秒watchdog、OOMなし、出力なしでraw errorを保全。holdout入力を改善判断には使っていない。次はeta更新後のdual分母不一致だけを新LUで再分解する仮説を別branchで検査する。11月19日UTC目安の全件開始と11月22日14:59UTC期限を守る。

## 2026-11-16 11:12 UTC Vec直結LU RHS

PR76をmain4556e45d6/FP527へマージ済み。次のvec-backed-lp-rhs FP5a6ccff29edba788は、Mat/Vec間の余分な確保とコピーを除き、非零column stride=nを維持する。根LPはbinkar20対4.03%、german3対5.32%、neos-860300 3対1.00%、neos-911970 10対1.25%短縮。全36対で時間以外のJSONが完全一致、保存72連続点も元MPS Fraction有効。219 Rust/2700合成/500 LP出力/数値32を成功。units初回はcheckerパス指定を落として未検証となり、同ELFでパスを正して100件成功、初回ログは保全。

全188×10秒は主解1改善/1悪化/186同値、dual1改善/0悪化/78同値/109比較不能。376出力/112整数点の独立監査成功。kasai1072846.63→1072833.43、loue22956→22966、neos-911970主解111.84→144.23の悪化は60秒でも残る。probe無効の対照は両版148.86で同じ。根LP自体の返却値はこの問題も同一だが、1秒probeの進み方とRINS固定集合が変わっている。原差を消さず、一般MIP品質改善とは言わない。60秒10問＋probe無効の22出力/18点も有効。LP費用を減らしたことを採用理由とし、この品質悪化を既知の制約として記録する。

主workspaceで再ビルドし、CPU41/10秒のkasai/neos-911970 native比較ではRustは全nativeより弱かった。kasaiの小さい主解改善は主workspaceでは再現せず1072846.63、neosは144.23。追加binkar6888.55/nidda14054.95/enlight37最適は従来品質を維持。11点と5Rust出力23切断・格子証明が有効。詳細はVEC_BACKED_RHS_RESULTS。PR77に原出力5435一次ファイル189,023,738bytesを保全、ELF6本とsamplingデータ2ファイルはローカルSHA参照、生成GLPKモデル2省略。全台帳照合成功。全240/3600秒vec-backed-rhs-final-20261116はcreate-onlyで全hash確認、全件は未開始。

分母不一致後の新LU再試行90afeはnidda主解悪化で棄却、8出力/4点が有効、source/原出力をsupporting-recordsに保全。perf cpu-clock samplingはPMU未対応、cycles:uは995samplesを取得し、コピー等の費用を確認。sampling時間を性能得点には使わない。

開発188だけのRust parser構造分類を完了（最適化呼出しなし、holdoutなし）。同係数の厳密行が多い例があり、次は元整数行の範囲支配で本当に冗長な行だけを除く仮説を検査する。係数一致の分類だけを削除証明とは呼ばない。nativeは11:11UTC 250/720、厳密最適112、エラー17で継続。11月19日UTC目安で最終Rust全件を開始し、11月22日14:59UTC期限を維持。

## 2026-11-16 12:35 UTC

PR77をmain b0b4e946d/FP5a6へ採用済み。行削減・Phase I即時/遅延価格付けは独立検査を通るが60秒主解悪化が残るため不採用。phase-one-delayed-pricing branchに6700一次ファイル216,510,391bytes（ELF7本はローカルSHA参照）を保存済み。main/全240snapshotは変更していない。

次のterminal-neighborhood branchでは数値停止後に限る主解専用の近傍探索を実装。v2 FP e0475a558f488f6e、Rust220/2700合成/500 LP出力/数値32が通過。Nidda 14054.95→13590.83、glass4 3466706233→2400021800を10/60秒対照で再現。10秒native比較でNidda主解のみ3者を超えたが下界は全3者より弱い。glass4は全3者より弱い。188対/全整数解監査とnative60秒比較を継続中。採用判断前に全結果を保存し、既存有効解の保持・大域状態不変を検査する。native全件は262/720・厳密最適114・error/無効解18、mas74 SCIPは公式相対有効だが絶対行違反0.087270506244でinvalid_solutionを保持。全240 Rust最終評価は未開始、11月19日UTC目安の開始と11月22日14:59UTC期限を維持。

## 2026-11-16 12:55 UTC

PR78をmain25048dd1e/FP134ef9e8e57efb90へ採用済み。全188対は主解3改善0悪化、下界同値79/比較不能109。全376証明/112整数点有効。primaryでもNidda13590.8271・glass4 2400021800・enlight37最適を再現、3整数点/3Rust証明が有効。terminal-neighborhood-final-20261116は全240/3600s/1thread/16GiB/CPU8–15 create-only、全SHA照合済み、本ジョブ未開始。

component-primal-diagnosticは未統合。20小例は結合主解が全列挙最適値と一致しGMP絶対/公式検査とも有効。初回2回のfree MPSは公式parser不受理のログを保全し固定書式で成功。cygnet120秒は6成分全て根LP時間終了、各5935–10300 pivot、整数点・下界なし、結合解なし。総wall124.82s、watchdogなし。採用せず停止原因を診断する。native269/720・厳密最適118・error/無効解18で継続。11月19日UTC目安の自作全件開始/11月22日14:59UTC期限を維持。

## 2026-11-16 13:28 UTC LP前の箱端点修復

PR79 candidate FPd5cc94b056af659a（測定試作c98）。全188×10秒は主解4改善0悪化184同値、下界同値79/比較不能109、最適件数両版1。376証明/116整数点がすべてGMP絶対・公式検査有効。cygnet220、sp97ar1233230307.52、sp98ar951499667.52、tbfp109.5486111118を従来点なしから取得。swath1の同主解/下界でtime→numericalという元差は保存する。

60秒はcygnetとtbfpの主解可用性改善が持続、sp97ar/sp98arは両版同じより良い点へ到達、co-100も両版点なし。10出力/6点有効。222 Rust（4 ignored）/2700合成/500 LP出力/数値32を通過。最終設定は既定有効化のみ。native比較16点/5Rust18切断も有効。cygnet主解は10/60秒Rust220/HS203/SCIP290/GLPKなし、sp98ar10秒はRust951.50M/HS971.20M/SCIP625.69M/GLPK9979.46M。下界はRustが弱く最適性未証明。

mainはまだPR78/FP134。PR79の最終設定確認と証跡保存を終えて採用する。全240最終実行未開始。次は既存目的格子が持つ原箱下界の結果表示、その後に箱端点主解からの根LP実行可能基底を別branchで検査する。native全件284/720・厳密最適128・error/無効18で継続。11月19日UTC目安で全件開始、11月22日14:59UTC期限を維持。

## 2026-11-16 13:58 UTC 既存目的箱下界の表示

PR80の全188対が完了。有限下界79→124、新規45・損失0、既存有限79件全同値。376証明/121整数点と65件の境界報告を原MPSで独立検査し全件有効。主解はH80なし→11250.99941、Kasai1072846.63→1072833.43という時間境界の差を保持し、表示機能の効果とは主張しない。終了状態は両版170time/11resource/1optimal/6numerical。最適件数/速度の改善ではない。

最終候補FPa11e46e075853620は既定有効化と旧期待値の更新。225 Rust（4 ignored）/2700合成/500LP出力/数値32、独立小例200/有限境界137/改ざん拒否548が成功。既定有効化初回はx∈{0,1}目的x・ゼロ予算の下界None期待が0になり1試験失敗、正しい下界0とtime_limit/LP0回、無効設定時Noneを確認して再試験225成功。初回ログ保全。最終対照/証跡保存後に採用する。

main b1bf71a87/FPd5と全240用root-seed-repair-final-20261116はcreate-only、最終本ジョブ未開始。PR79採用後primary6点/6Rust18切断の再確認とsnapshotを194一次ファイル8,981,317bytesへ保存済み。次は箱端点主解からの実行可能LP基底を別branchで検査する。まだ未実装。11月19日UTC目安の全件開始・11月22日14:59UTC期限を維持。

## 2026-11-16 15:30 UTC LP開始基底案を不採用

PR80をmain5792ef5ac/FPa11e46e075853620へ採用。primaryの3整数点/4証明が有効。objective-domain-report-final-20261116は240件3600秒1thread16GiBのcreate-onlyで本ジョブ未開始。

incumbent-lp-warmは不採用。V3全188対は主解2改善2悪化184同値、有限下界124→126。N3div36は168800→197400、下界116000→114600。neos-911970は111.84→144.23だが888変数で適用範囲外、時間差として区別。全376証明/120整数点と追加46証明/46整数点が独立検査有効。V4のcold-firstはN3の高速LPを維持したがsp98ar60秒726483084.8→776467506.56の悪化が残る。V3の300秒でも同じ悪化。主解基底の変更をmainへ入れない。

300 LP×2系列の各600点が独立目的値/有理数実行可能性検査を通過し、229 Rust/2700合成/500 LP出力/数値32も成功。未採用ソースと8943一次ファイル303326806bytesを保存、ELF6本はSHA参照。次は探索順を変えず、正の被覆行の集約から元問題の証明器で下界だけを得る案を診断する。全240最終実行は未開始、11月19日UTC目安の開始と11月22日14:59UTC期限を維持。

## 2026-11-16 16:10 UTC 正の被覆行の報告下界

PR82候補FP61998ca852be9e2e。全188×10秒の同一バイナリenv0/1（測定V2 FP67638bfbd6bdf037）は主解188同値、既存有限下界5改善119同値0悪化、新規有限6/損失0で124→130件。全376証明/122整数点が独立検査有効。最適件数両版1、状態両版171time/11resource/1optimal/5numerical。探索・枝刈り・終了判定は変更せず、0.25秒/5%上限の前計算を結果の下界にだけ使う。

cygnet60秒Rust220/149.48、HiGHS203/0、SCIP290/0、GLPK点/下界なし。3整数点とRust証明有効。総合優越や新記録ではない。200小例/1000改ざん拒否、2700合成、500 LP出力、数値32例（18有効/14未解決）を通過。最終既定有効化の旧単独試験は下界None期待で1失敗し、機能隔離とnode_limit保持の組合せ試験を追加して228 Rust（4 ignored）成功。最終4出力/4点も全有効。

別branchの非交差行下界はcygnet170を10/60秒で証明し、主解220維持、追加計算約0.020秒。公開JSONの重複列を再検査するV2 FP422453d2603d81f5で188対を実行中。300証明有効/900改ざん拒否。さらに独立Repair+目的上限制約はwhole10秒216、6成分各5秒（JSON読込含むwall約58秒）210を得て元MPS厳密検査有効。まだ未統合で比較成績には含めない。次は非交差行下界の全件検査と、成分読込を省いた主解改善診断。

main7587fb873/FP a11e46e075853620、最新全240 snapshotはobjective-domain-report-final-20261116 create-only。本評価未開始。native比較315/720・厳密最適139・error/無効18で継続。11月19日UTC目安の自作全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 16:30 UTC 非交差行の厳密整数下界

PR83最終候補FP7e7a368cf545ac9d、230 Rust（4 ignored）成功。開発188×10秒の同一バイナリ対照（測定FP422453d2603d81f5、均一乗数は両側有効）は主解188同値、有限下界3改善127同値0悪化、有限130件維持/新規損失0。glass-sc4.255→8、cygnet149.484→170、seymour1 35.685→359。状態・最適件数は同じ。376証明/120整数点と追加20証明/18点すべて独立検査有効。最終実装の元MPS証明300/改ざん900拒否も成功。

native10秒seymour1 Rust460/359、HS413.9006/405.7281、SCIP423.2322/407.1796、GLPK413.0815/406.0253。glass-sc Rust28/8、HS23/15、SCIP23/14.5316、GLPK23/15。両問題はRustが全3者より弱い。cygnet60秒Rust220/170、HS203/0、SCIP290/0、GLPK点/下界なし。全11native比較整数点の厳密・公式検査が有効。総合優越/新記録はない。

独立目的上限制約の修復は短時間whole0.5秒218、成分60秒208まで改善し全点有効。短時間7点も全検査有効。別ワークツリーmip-objective-repairのroot-objective-repair-v1 FP10e763eef59eca62は、根の既存自作解にだけ単位費用の目的上限制約を加える試作。全231 Rust成功後、追加を含む統合2試験も成功。原解保持・node_limit/LP0回の保持を確認。cygnet10秒の対照で220→218、下界170同値、追加計算0.515773291秒を確認し、他の選択対照と独立点監査へ進む。まだ未採用。

mainad29d8faf/FP61998ca852be9e2eのprimary3点/3証明は再現/有効。row-ray-bound-final-20261116は全240/3600s/1thread/16GiB create-onlyで全SHA照合済み、本ジョブ未開始。native比較を継続し11月19日UTC目安の全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 16:55 UTC 目的上限制約による主解修復

PR84候補FP9faaaa51331767bb、232 Rust（4 ignored）通過。全188×10秒の同一バイナリ対照（測定FP10e763eef59eca62）は主解1改善0悪化187同値、有限下界130件全同値/新規損失0、最適件数両版1、状態両版171time/11resource/1optimal/5numerical。376証明/120整数点、選択10証明/10点、最終4出力/4点すべて厳密・公式検査有効。

cygnetは10/60秒とも220→218、下界170同値。追加計算0.515773291秒、0.5秒期限のわずかな超過も実測を保持。暫定目的制約の不能を大域判定に使わず原解を保持する。独立300小例の元MPS絶対検査/全列挙も成功。native10秒はRust218/170、HS203/0、SCIP290/0、GLPK点/下界なし、3点の厳密・公式検査有効。初回のディレクトリ/Python環境不足をerror/ログで保存して環境を揃え再測定。総合優越/新記録は未達。

劣勾配下界案は5設定で170から改善なし、独立有理数検査全有効。比較用連続LPも60秒で未解決、raw objective170.3127は上下界に使わない。33ファイル37996bytesを保存し未採用。v1 ELFはsource/hashのみ、v2 ELFは実行前SHAと一致する実物を保持。主ソルバーへ入れない。

次は既存box探索の目的下降を既存整数点へ適用するroot-primal-descent branch。アルゴリズムは新しい候補の元問題検査と有限目的の保持を必須とし、目的桁落ちが許容精度内と証明できない場合も元点へ戻す。既存234 Rust後に追加ガードを含む3試験が成功。まだ独立診断でB&B未接続。

main eeb0c692e/FP7e7a368cf545ac9d、primary3点/3証明が再現して全有効。全240設定row-packing-bound-final-20261116はcreate-only/全SHA照合済みで本ジョブ未開始。147一次ファイル2901953bytesを保存。11月19日UTC目安の全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 17:45 UTC 根の主解への目的下降

PR85候補FP2e6fa83a900b8a89、235 Rust（4 ignored）/元MPS300例/最終4点4証明が全有効。全188対（測定FPca13fe57dc00c18a）は主解1改善1悪化186同値、有限下界130件全同値、376証明/120点有効。Kasai改善とeil33-2悪化は本処理が呼ばれない適用外で、原因を断定しない。eil33-2は10秒1116.990809→1208.773082、60秒両版1116.990809同値。状態before171time/11resource/1optimal/5numerical、after170/11/1/6を保持。

cygnetは10/60秒とも218/170、局所処理0.52076→0.10198秒/0.50520→0.10272秒、LP pivot1264→1313/7350→7393。全MILP速度や最適件数向上は主張せず、原点保持と目的誤差ガードを維持する局所効率化として採用を検討する。選択/長時間12点12証明も有効。5121一次ファイル209795111bytes、ELF4本はSHA参照に保存。

次の費用付き修復は別worktree/branch priced-root-repair、採用済みmainを基点に未採用の下降には依存しない。試作FPc36f4320f532a0ff。重み0は旧順位、重み1はgain/(1+cost/正の目的係数中央値)。0.1秒/100M workで、元点なしなら検査済み点を、元点ありなら誤差幅を超えた改善だけを保持。200小例と4096変数のnode_limit/LP0回/100→2の統合試験を通過。SP97 1233230307.52→894096878.72、SP98 951499667.52→664376405.28、N3div36 168800→142800を10秒で確認。60秒SP97同値、SP98の改善維持。188対/独立監査中、まだ未採用。native10秒はSP97主解GLPK超え、SP98主解HS/GLPK超えだが下界は全3者より弱い。

main129090b8b/FP9faaaa51331767bb、primary cygnet218/170・enlight37・Nidda13590.8271を再現し3点3証明全有効。全240設定root-objective-repair-final-20261116はcreate-only、本ジョブ未開始。148ファイル2790685bytesに保存。11月19日UTC目安の全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 18:00 UTC 費用付き根の修復

PR86候補FPca7c7bbcdf8ee25b、237 Rust（4 ignored）成功。開発188×10秒の同一バイナリ対照（測定FPc36f4320f532a0ff）は主解4改善0悪化184同値、有限下界130件全同値、最適件数両版1、状態両版170time/11resource/1optimal/6numerical。376証明/120整数点有効。N3div36/SP97/SP98の3改善で処理が動き、Kasaiの差は処理なしの適用外として保持。

10秒N3div36 168800→142800、SP97 1233230307.52→894096878.72、SP98 951499667.52→664376405.28。60秒N3とSP98の改善維持、SP97は両版889668772.16同値。下界は全件同値。選択8/長時間6/最終8出力と整数点が全件厳密・公式検査有効。独立診断12点も全有効、重み0旧順位一致/200小例/4096変数のLPなし100→2でもnode_limit保持を検査。

native10秒N3 Rust142800/116000、HS137400/120344.71、SCIP397200/0、GLPK270800/123800。SP97 Rust894096878.72/253899438.08、HS813180828.8/654469696.71、SCIP779179182.72/654045092.66、GLPK1119772686.08/653338970.70。SP98 Rust664376405.28/211240207.21、HS971199029.6/526299546.97、SCIP625690196.32/525523370.20、GLPK9979463169.44/526010283.34。全12点の厳密・公式検査有効。主解で上回る比較はあるが下界の弱さと劣る比較も保持し、総合優越/新記録は主張しない。4725ファイル261314508bytes、ELF/生成GLPK入力6本はSHA参照に保存。

PR85はmain2b9744189/FP2e6fa83a900b8a89へ採用。primary3点3証明全有効、root-primal-descent-final-20261116は全240/3600s/1thread/16GiB create-only、全SHA確認済み、本ジョブ未開始。148一次ファイル2795207bytesを保存。次は価格重み0.3/3の小規模追加診断、良い点を保つ共通予算内portfolioの可否を確認する。まだ未採用。11月19日UTC目安の全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 18:12 UTC 共通価格portfolioの対照開始

PR86はmain f814b53aeへ採用、primary FPca7c7bbcdf8ee25bの4点4証明が絶対・公式検査で全有効。n3div36=142800/116000、SP98=664376405.28/211240207.2069437、cygnet=218/170、enlight_hard=37 optimal。priced-root-repair-final-20261116は全240/3600s/1thread/16GiBのcreate-only、SHA確認済み、本ジョブ未開始。166一次ファイル7127671bytes、ELF4本はSHA参照として保存。

次のportfolioは共通重み[1,0.3]を同じ0.1秒・合計100M workに収め、既存最良点を保持する。実験flag既定false、測定FP886ac0f0fca7f632を実行前固定。238 Rust（4 ignored）成功。n3div36の10秒は142800→134600、下界116000同値、局所phase0.01364→0.02651秒。188対・独立監査・3件60秒・同条件native10秒を開始。最初の選択driverは拡張子なしの名前でKeyErrorとなり本ジョブ0件、失敗manifest/logを保存して正しいmanifest名で再実行した。全体優位・新記録は未達。

## 2026-11-16 18:27 UTC 共通価格portfolioの採用判定

試作FP886ac0f0fca7f632の全188×10秒は主解1改善0悪化187同値、有限下界130件全同値、58件比較不能、最適件数両版1。状態は両版171time/11resource/1optimal/5numerical、376証明/120点すべて独立GMP絶対・公式検査有効。N3div36 142800→134600の改善は60秒でも維持、SP97/SP98は10/60秒とも同値。選択8点8証明、長時間6点6証明、native12点3証明が全有効。N3の主解は同条件native3者を上回るが下界はHS/GLPKより弱く、全体優位・新記録は未達。

既定trueの最終FP523fc50c7f8550bbは238 Rust（4 ignored）成功。最終4対照を実行中。合計100M work/共通0.1秒と既存良点保持を維持し、総合速度向上は主張しない。PR86 primary追試4点4証明と全240create-only snapshotは166ファイル7127671bytesに保存済み。

次の目的cut一般費用版は別worktree priced-objective-root、採用済みmain f814b53aeを基点に試作。FPfe519f9af5a3bbb1で238 Rust/200小規模全列挙照合成功。0.5秒phaseでSP97 894096878.72→884197292.16、SP98 664376405.28→662760404.64を統合10秒でも再現、N3/TBFP同値。選択8点8証明有効。独立診断2秒/10秒は元MPS全点有効だが全体10秒の比較ではない。188対と60秒/native/元MPS監査を継続し、まだ未採用。

## 2026-11-16 18:40 UTC 非一様費用の目的cut修復

試作FPfe519f9af5a3bbb1の全188×10秒は主解2改善0悪化186同値、有限下界130件全同値、比較不能58、最適件数両版1。状態両版171time/11resource/1optimal/5numerical。376証明/120点が全件GMP絶対・公式検査有効。SP97は10秒894096878.72→884197292.16、60秒889668772.16→862015925.12。SP98は10/60秒664376405.28→662760404.64、下界は同値。選択8点8証明/長時間4点4証明/native8点2証明も全有効。

採用済みportfolioを併合し、両方既定有効の最終FP a53556b6ad0b400aを実行前固定。239 Rust（4 ignored）成功。適用5件の10秒最終対照とN3/eil33-2の60秒相互作用確認を開始。追加探索の最大5%/2秒、16 seed、合計100M workと既存良点保持を維持し、目的制約の失敗を大域不能に使わない。全体優位・新記録は未達。

PR87はmain b4f36aa4a/FP523fc50c7f8550bbへ採用。primary N3=134600/116000、SP98=664376405.28/211240207.2069437、cygnet=218/170、enlight37 optimalを再現し4点4証明全有効。priced-portfolio-final-20261116は全240/3600秒/1 thread/16GiB create-onlyで全SHA確認、本ジョブ未開始。166一次ファイル7128390bytesを保存。native全件は継続、11月19日UTC目安の自作全件開始と11月22日14:59UTC期限を維持。

## 2026-11-16 18:50 UTC 改善系列への探索継続

PR88をmain be31401ba/FP a53556b6ad0b400aへ採用。primary N3=134600/116000、SP97=884197292.16/253899438.07998765、SP98=662760404.64/211240207.2069437、enlight37 optimalを再現、4点4証明が全件元MPSのGMP絶対・公式検査有効。priced-objective-root-final-20261116は全240/3600秒/1 thread/16GiB create-onlyで全SHA照合、本ジョブ未開始。167一次ファイル6176108bytesを保存。

次のpriced-objective-continuation branchは16 seedで改善が出た系列だけ最大64へ継続し、改善0の系列は16回で停止する。元公開APIと原点保持/共通5%・2秒/総100M work/証明guardを維持。実験既定false、測定FP6aa6bb56ffcb27e3を実行前固定。既存239 Rustと追加の継続条件・200全列挙照合を含む2試験が成功。小規模で成功系列64回/失敗のみ16回を確認。10秒選択比較は現時点で同値、60秒/全188対/独立監査を継続中。まだ未採用、全体優位・新記録は未達。

## 2026-11-16 19:10 UTC 成功系列の予算内継続

測定FP6aa6bb56ffcb27e3の全188×10秒は主解1改善0悪化187同値、有限下界130件全同値、58件比較不能、最適件数両版1。状態両版171time/11resource/1optimal/5numerical、376証明/120点全有効。eil33-2の差1208.773082→1116.990809は当該phaseが両版16回/改善0/同workのため効果と断定しない。選択10秒は5件すべて主解・下界同値、10点10証明有効。

60秒SP97 862015925.12→838597508.8、SP98 662760404.64→658266817.76、下界同値、4点4証明有効。native60秒は自作の主解はGLPKより良いがHS/SCIPより劣り、下界は全3者より弱い。native8点2自作証明全有効。全体優位・新記録は未達。追加30秒独立診断765919156.8/613700763.36は2点全有効だが別条件と明記。摂動量4組×2モデルは既存より改善せず不採用、8点全有効。

最終FP12ee05629152fe08は240 Rust（4 ignored）/200小例×2APIの400出力全列挙照合が成功。最終10秒5対は品質全同値、10点10証明有効。60秒でも2件の改善を再現、最終4点4証明監査中。資料固定後PRへ採用する。次は最初の既存予算内に改善した場合だけ長時間枠の残余を使う仮説を別ブランチで検証する。

## 2026-11-16 19:53 UTC 長時間予算配分と非一様下界

PR90をmain 5fcdadd54/FP0331d37b4c5a454dへ採用。全188×10秒は主解188同値/有限下界130同値、376証明/120点全有効。60秒SP97=826960232.0、SP98=655504854.72、300秒試作FPは789673233.28/632800096.0へ改善、下界同値。最終241 Rust/200小例×3API600出力全列挙照合、最終10秒10点10証明/60秒4点4証明が全有効。4594一次ファイル248524377bytes、ELF/GLPK入力5本はSHA参照。LP行順位案は1改善2悪化2同値で不採用、10点10証明全有効、448ファイル9871597bytesを保存しLP本体へ入れなかった。

primary60秒はSP97=826960232.0/652580162.4662657、SP98=655504854.72/524472090.3097343、N3=134600/116000、enlight37 optimalを再現し4点4証明全有効。priced-objective-budget-final-20261116は全240/3600秒/1 thread/16GiB create-onlyで全SHA照合、本ジョブ未開始。167一次ファイル6178636bytesを保存。native全体は366/720、厳密最適161/エラー19。追加エラーは112-glpkのwatchdog3660.513秒/exit−9/結果JSONなしで、通常time_limitへ置換しない。

非一様被覆乗数の2秒/短期独立診断14件が元MPS有理数照合を通過。次のpriced-row-subgradient branchは採用済みPR89を基点とし、FP6c7f48c378f34a71を実行前固定。最大0.1秒の75%を探索へ配分し残りを区間証明構築に確保、report-onlyで既存下界のmaxを維持する。既存241 Rust後に200小例全列挙/4096変数node_limit・LP0回・主解101保持の2試験成功。元MPS oracle200出力全有効、162新乗数証明、648改変全拒否。

選択10秒5件は主解同値、SP97 dual253899438.08→539059967.19、SP98 211240207.21→399725206.96、他3件の最終dualは同値、10点10証明全有効。188対ではfast0507にも74.5→154.9928の改善が出た。全188/逐次独立監査、SP97/SP98の60秒対照、native10秒を継続中。native2モデル8点/自作2証明は全有効、まだ未採用。全体優位・新記録は未達、11月19日UTC目安の自作全件開始と11月22日14:59UTC期限を維持。
