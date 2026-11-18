# 最終評価の読取り専用集計

report_benchmark.pyは最適化を呼ばず、モデルファイルも読まない。正式240問の測定と独立検査が終わってから使用する。作成時の検証は人工データのみであり、検証群52問の個別結果を開発判断に使っていない。

同じseconds、16GiB、threads、cores、読書き猶予、検査期限、manifest SHAのrunだけをまとめる。各summaryの問題名・solver・入力SHAとconfigを照合し、結果JSONのSHAも照合する。同一問題名の入力SHAがソルバー間で違えば停止する。すべてのsummary SHAとconfig SHAを出力する。snapshotと共有libraryの監査は別途、凍結時と起動前の既存手順で行う。

primary strict検査に合格した点だけをprimalとして使い、GMPの再計算目的値をFractionで読む。公式検査は別列。実行失敗、watchdog、OOM、欠測を解・境界の比較で得点させない。検査合格の点が残る非最適終了は主解として数えるが、時間集計は上限秒を課す。不合格/未検証のcut証明と、向きが矛盾するdualをusable dualから除く。共通gapはabs(P-D)/max(1,abs(P))。

最適として数える条件は、正常実行、status optimal、primary合格点、usable dual、共通gap<=1e-7。表示名はdeclared_optimal_with_valid_primalとし、独立した厳密最適性証明と呼ばない。既存runnerのverified_optimalフィールドを無条件に信用して集計しない。厳密な全探索証明の独立検証を行ったとの主張もせず、そのフラグはfalseである。元status/solver_status、各点・境界・gap、原検査結果へのsourceを残す。

時間は最適条件を満たした場合だけsolver報告秒を使い、上限でcapする。他は欠測・失敗も含め上限秒、shift10の幾何平均。solver時間とexecution wall時間を別列にする。pairwise比較は共通問題ごとにprimal/dual別集計し、abs差<=1e-8*(1+max(abs(a),abs(b)))を同等とする。片方だけ値がある場合と両方値がない場合を勝敗と分離する。全MIPLIB2017 benchmark-v2が最小化である入力監査に対応して既定min、一般の一様な最大化群には--maximizeを明示する。

ディスク障害の再測定は、事前固定したrecovery runを--replacementに明示する。同じ問題/solver/入力SHA/CPUの場合だけ置換し、悪い結果でも新結果を使用する。元summary全体をreplaced_originalsへ保持し、同じcaseの複数置換を拒否する。qualityで選んだ置換をツールが正当化するものではない。時刻だけで選んだ1件の既存方針を使用する。出力は新規作成のみ。

人工データ8テスト：2^53超の正確な目的値、失敗/未検査点、タイムアウト点と全上限課金、最小化/最大化の不正境界、非有限境界、比較方向/片側欠測、悪化した事前固定置換と元結果保持、SHA改ざん、資源/CPU不一致、欠測保持、重複置換拒否、shift10計算。実データの集計はこの実装を固定してから行う。
