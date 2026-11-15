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
