# 偶奇等式の一意な整数解を直接証明

私有の±2整数補助変数を持つbinary整数等式をGF(2)へ写し、binary列がfull rankなら元整数解が一意であることを証明する。行ごとのi128復元と元十進境界の内側検査で実行可能性を確認し、元目的係数の外側区間から目的boundを計算する。問題名/既知解は使わない。構造・精度・一意性が確認できなければ通常MILPへ戻り、この経路は実行不能を宣言しない。

実装の上限は2048行/2048binary/4096変数、消去2000万word更新、残り時間の内最大0.05秒。MIP_PARITY_EQUALITIES=0/1。候補は既定ON、FP527633393c003557。Pythonの独立検査は元MPSをFractionで読み、Rustの列順RREFとは異なるhighest-bit streaming消去で階数と一意な復元点・目的・boundを照合する。benchmark_validateで証明失敗をstrict不合格にする。通常のGMP絶対/公式点検査も必要。

## 開発188件、同一ELF・各10秒

1 thread/16GiB、before=flag0/after=flag1、同じCPUで交互に測定。全376実行完了、ドライバーエラー0。主解1改善/0悪化/187同値、dual1改善/0悪化/78同値/109比較不能。enlight_hardのみが発動し、主解なし/dual16.4583333333からoptimal37/dual36.99999999999989となった。最適判定は元MPSからの独立一意性証明を含め検査済み。

状態はbefore 172 time_limit/11 resource_limit/5 numerical_error、after 170/11/6＋optimal1。swath1は主解448.882839/dual334.496858が同じで、10.001秒付近のtime_limit→numerical_error差が残る。別の同CPU0/1再測定では両方time_limitで同品質。最初の差を消去/補完しない。SGM(shift1、未解決10秒penalty)は10→9.8605954266だが、1件の解決による値であり全体速度優位の証拠ではない。

376出力のFraction切断/目的格子証明と111整数点のGMP絶対・公式検査がすべて有効。切断検査を全探索木の再生とは呼ばない。今回の偶奇最適判定は全元行・整数domain・独立階数検査から直接証明している。

## 60秒対照とnative実測

enlight_hardは無効60秒でも主解なし/dual16.4583、有効は同じ最適37。timtab1、sp150x300d、binkar10_1、neos-860300、germanrr、niddaの主解/dualは両設定で同じ。swathの10秒再測定も含む16出力/13整数点は独立検査成功。

主workspaceで同じFPを再ビルドし、CPU41/10秒/16GiB/1threadで測定したenlight_hard:

|solver|主解|dual|状態|内部秒|
|---|---:|---:|---|---:|
|Rust|37|36.99999999999989|optimal|0.000153376|
|HiGHS 1.16.0|なし|36|time limit|10.0141|
|SCIP 10.0.2|37|37|optimal|0.003572|
|GLPK 5.0|37|25|feasible limit|10.0011|

同じ既知最適値の再現であり新記録ではない。起動・読込の違いがあるため内部時間と全process時間は原出力で分離する。事前worktree比較の最初のGLPKはdriverパス不足で起動失敗し、生ログを残して正しいdriverを指定し4 solverとも再測定した。主workspaceの比較は通常パスで全件完走。主workspaceの追加binkar/niddaは従来と同じ品質で、5保存点・3Rust出力23切断/格子証明がすべて有効。worktree比較の5保存点も全件有効。

## 正しさ・保存・次の作業

219 Rust回帰（300小問題の全列挙を含む）、CLI生成200問題の全列挙照合（175最適点をGMP検査、25実行不能を列挙確認、90一意性証明・270改変拒否）、通常合成2700、根LP300設定500出力、数値32が成功。数値32は18有効点と14精度未解決であり全32点が実行可能という意味ではない。小数目的、min/max、offset、負の私有整数、元小数が整数へ丸められた不正適格性、階数不足、矛盾、期限切れを確認した。

一次データはartifacts/parity-equality-evaluation、未採用数値回復/一般整数pumpはartifacts/heuristic-recovery-negative-evaluation。凍結ELFはローカル保持しSHA/元ソース/build-infoで参照。relocation-ledgerで保存先のbyte SHAを照合する。全240/3600秒/1thread/16GiBのparity-equality-final-20261116はcreate-only、全snapshot hash照合済み。自作全240実行は未開始、全体native優位性は未達成。native720件の凍結キャンペーンを続け、最終自作全件を11月19日UTC目安で開始する。

次の独立仮説は、dual ratioの逆積と正方向の分母が食い違った場合に、eta更新が残っているときだけ新LUへ再分解して同じ節点を再試行すること。許容誤差を緩めず、総pivot/flip/deadlineを引き継ぎ、改善がなければ未採用記録へ戻す。
