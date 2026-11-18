# pump終端点を元行で短く修復する候補

main130で打切り点の単純丸めを主解として回収できるが、初期188比較では17点中16点は拒否された。次候補は、単純丸め失敗後だけ既存のRust座標修復 search_adaptive_incremental を使う。外部最適化は用いない。MIP_PUMP_TERMINAL_REPAIRの既定OFF、MIP_PUMP_LIMITED_ROUNDINGが有効な時だけ適用。元変数写像後の丸め前点から開始し、root変数bounds、元行の既存candidate_checkを保持する。

局所deadlineは残り全体時間の1%と100msの小さい方、work予算500000、固定汎用乱数seed。既存修復の内部scanによる超過は実測時間に残す。全体期限を過ぎた結果は採用しない。LP予算・乱数軌跡・制約・許容差・境界・枝刈り規則は変更せず、返すのは可行主解のみ。回収点に伴う通常incumbent枝刈り以外の証明情報は使わない。

事前選択は旧候補2dの188afterのlimited_points>0全17問。成否・目的値で選ばない。同じ新凍結CLI OFF/ON、各10秒/16GiB/1thread、CPU44/49で測定し全点元GMP/公式、全proofを独立検査する。CPU40/47/50の自作1198reseed、46の長round1時間とは独立に実施。利益がなければ採用しない。後続188比較では時間配分による損失も保存しholdout52を調整に使わない。main130の4post検査と候補/primaryのmcsched値の変動は別archiveで引継ぐ。

初期pilot完了:17問34出力、全34proof/10点fresh GMP公式valid。主解14同等/2改善/1悪化、dual16同等/1比較不能。修復実行16問、受理3問。drayage None→645963.1709289545、graph20 None→−7、h80 11228.774227→26428.164941（悪化）を全保持。h80はCPU43同一凍結3pairsでOFF全None/ON全26428.164941、6proof/3GMP validで、初期損失を置換しない。修復workは初期全行scan分でbudget500000を超える例(co-100 1998004)があり、soft work予算として消費をそのまま記録する。局所追加時間はミリ秒単位で、次の188問比較を同じ凍結CLI/明示flag0/1で開始済み。

10秒native比較も全6点GMP valid。drayageはHS1.16.0が101282.64701843262/LB同値で1.474759579秒最適、SCIP10.0.2が136525.13797760004/LB100121.99054718018/10.000627秒、GLPK5が812730.2231597898/LB23484.125649987134/10.196435379秒。graph20はHS−8/LB−29/10.003687859秒、SCIP−9/LB−9/4.979922秒最適、GLPK−5/LB−37/10.243033764秒。本候補は両問GLPKより主解が良いがHS/SCIPに劣り、全体優位は未達。native点はRustへ渡さず、CPU43/45/16GiB/1threadで比較。

初期188は376fresh proof/133点fresh GMP公式valid、primal183同等/4改善/1悪化、dual173同等/1改善/1悪化/13比較不能。drayage/graph20/h80は新修復でNoneから可行点を得た。blp-ic98の改善とneos-911970の129.38→140.59悪化、fast0507の下界166.600693→166.291933悪化、seymour1下界改善は新処理未実行。追加neos3pairsは1組111.47→140.59悪化/2組140.59同等、fast3pairsは主解全199でdual1同等/2悪化(166.781069→166.600693)。12点GMP/12proof valid、全pump.calls=0で時刻依存変動を保持し初期結果は置換しない。既定ON coreb3da9b/FP93bddc12fc843734/CLIcec43で360Rust4ignored後、環境変数未指定の188問比較を開始。source差は既定boolだけ、他の数学/時間配分は不変。

既定ON再比較完了:376fresh proof/133点fresh GMP公式valid。primal184同等/4改善/0悪化、dual173同等/0改善/2悪化/13比較不能、両182time_limit/2optimal/4numerical_error、finite175。新処理由来はdrayage645963.170929/graph20−7/h8026428.164941の3点を再現。mcsched312590→309595の改善はterminal_repair_calls=0で時刻打切り変動。seymour1 dual394.934512→394.789578、sp97ar595791964.39835→592005695.175724の悪化も新処理未実行として保持。追加binary200環境変数未指定は全成功。初期h80/neos損失・反復での損失も残し、既定ONによる早期可行点を根拠に採用を提案する。LP予算・全体期限・許容差は変えず、新修復は既存全体時間を消費する。
