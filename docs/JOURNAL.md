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
