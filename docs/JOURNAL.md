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
