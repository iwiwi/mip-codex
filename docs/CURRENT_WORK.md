# 進行中の作業（2026-11-15 02:08 UTC）
ユーザー指示により別の `codex --yolo` PID 10041 をSIGTERMで終了。kill(pid,0)がProcessLookupErrorとなり終了確認済み。以後こちらの継続セッションのみで開発する。

現在の実装: src/model.rs MPS読込、src/lp.rs 二段階有界変数改訂単体法（Rust最適化、faerの汎用疎LUのみ使用）、src/mip.rs DFS分枝限定、CLI。別プロセスからの更新後にAPIを統合し、cargo test 4件と全列挙300問が成功。先行する密単体法実装は別プロセスに置換されたため現状の実装とは区別する。
scripts/check_solution.py はHiGHSをMPS読込専用に使用する独立検査器。toyで最大違反0、目的値誤差0を確認。CLIは変数名も出力する。

進行中の実ジョブ:
- MPS全240問読込: exec session 47966。logs/mps-inspection.jsonへ完了時に記録。実行中のinspect子プロセスを確認済み。旧パーサをコンパイルしたdebugバイナリで始めたため現行パーサの全件検証は別途必要。
- cargo build --release: exec session 87042。初回依存ビルド中。

未完了: 比較実験ランナー、比較版・設定・バイナリハッシュ固定、MIPLIBの実走、PR作成・main統合。MIPでunbounded relaxationはunbounded宣言せず未解決扱い。
使用量集計 scripts/ai_usage.py、docs/ai-usage.json は集計時点のスナップショット。セッション累積最終値のみ加算。
入力は docs/benchmark-manifest.json に240問のハッシュと分割を保存。別プロセスの scripts/fetch_miplib.py と data/manifest.json 等も存在。二重の取得経路は統一が必要。data/benchmark と data/raw は大容量なのでgitへ追加しない。
次: リリースビルド終了確認、開発側の小さな問題で1/10秒の4ソルバー比較、解検査、乱数連続LPの外部比較、基盤PRを作成。

02:10 UTC thread 01a1dcec の追加作業: scripts/test_lp.py を追加し現行疎LPをHiGHSと乱数比較中（exec 19072、logs/lp-differential.log/.exit）。初回の不一致はテスト生成器がFR後にLO/UPを重複定義しHiGHSが後続境界を無視したことが原因。生成器を非重複境界に修正し再実行中。src/main.rsとsrc/mip.rsは現行のまま保持。この系の担当はsrc/lp.rsとscripts/test_lp.pyの検証・修正のみとし、比較ランナーとPRは他の継続系へ譲る。

02:13 UTC / thread 01a1dcec: LP 1000問の差分検証が全件成功（126 infeasible,583 optimal,291 unbounded）。250問/最大100変数150行のストレス検証exec88652進行中。eta100回更新と新規LUのFTRAN/BTRAN一致テストをsrc/lp.rsに追加。境界意味の外部照合で、LO 0+UP -1で下限が勝手に-infになるパーサバグを修正し回帰テスト追加（HiGHSと同じく既定下限0を保持）。MPS慣習には負のUP時の既定下限に方言があるため評価でのモデル一致も要確認。main/mipは変更していない。

追加検証の現状（この追記が古い1000問全件成功の記述に優先）:
- 小さいピボットを無視した有限LPの非有界誤判定を防ぐガードをsrc/lp.rsへ追加。tiny coefficient回帰テストあり。真に非有界でも丸め誤差の微小ピボットにより数値失敗として止まることを許容し、性能の成功に数えない。
- 最新小規模LP: 有限境界付き1000問=839 optimal+161 infeasible、未解決0、不一致0。自由変数あり1000問=583 optimal+126 infeasible+270 unbounded_relaxation+21 numerical_failure。誤った終端判定0だが未解決21なので全件成功とは呼ばない。results/correctnessに要約保存。
- 大きめ250問の初回はHiGHS既定単体法がcase95,105でUnknown。HiGHS IPMとSCIPが両方infeasible、自作もinfeasible。scripts/test_lp.pyはこのときIPMへ照会し回数を記録。最新再実行exec70889（logs/lp-stress-safe.log/.exit）。
- LPテストの --allow-unresolved は明示的に数値失敗等を記録するモード。誤答は常に失敗、未解決を解決に数えない。--bounded は標準の全件解決確認。
- results/lp-failuresには初期生成器の重複境界による不一致も残る。再現時には保存MPSを使い、同じseedだけで初期生成器を再現したとみなさない。

02:15 UTC: 大きめ250問は159 optimal+34 infeasible+57 unbounded_relaxation、不一致0・未解決0、HiGHS IPM fallback2で終了。ただし実行中にdebugバイナリを再ビルドしたため厳密な固定版結果として採用しない。scripts/test_lp.pyをバイナリコピー・SHA256記録方式にし今後のテスト版混入を防止。
- 全体が微小な方向ベクトルにも相対ピボット閾値を適用。1e-12係数により最適値-1e12となる回帰例はoptimalを要求し通過。
- 独立検査器の行活動・目的値の集計をmath.fsumへ変更し、名前順入れ替えのtoy解が最大違反0で通過確認。
- 比較ランナーも実行バイナリが途中のcargo buildで変わらないよう、実行開始時にコピーしてその固定コピーを呼ぶ必要がある（現在のbenchmark.pyはtarget/releaseを直接参照）。

02:17 UTC / thread 01a1dcec: 疎LP側の編集をいったん停止し、基盤PR用に現行ソースを安定化した。現時点unit7件、全列挙300問、固定コピーbinary SHA256=12046189993c1198ee3999cba73bfafdf380a735812f861f392239e222a22a2dに対する有限境界LP1000問（未解決0、不一致0）、自由変数LP1000問（未解決21、不一致0）の要約をresults/correctnessへ保存済み。releaseバイナリはこのdebug版と別なので、性能と正しさの版を混同しない。
残課題: tiny-blockerガードは安全側に止まるため、test_continuous.pyのstrict終端一致テストが新releaseで未解決を検出する可能性がある。数値失敗は成功に数えず報告するか、rayの検証/再計算を実装して解決すべき。現在の自由変数テストは明示 --allow-unresolved 指定時のみ数値失敗を報告して継続する。
