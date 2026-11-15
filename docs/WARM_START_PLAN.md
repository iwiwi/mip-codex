# 次段階: ノード間基底再利用 (未実装の仮説)

現状は各ノードで二段階primal simplexから開始する。疎版でLP処理は速くなったが、親の最適基底を捨てている。基底・非基底の上下限状態・LU factorを共有し、子の境界変更後に基本解を再計算してbounded dual simplexで修復する。

先に列番号を安定化する必要がある。現在はRHSに応じて人工変数を追加するため、ノードごとにslack/artificial番号が変わる。全行に正負の単位人工列を決定的に配置し、phase2で両方の上限を0にする方式を検討する。元の列/費用/行構造をfingerprintで検査し、自由変数の変換が変わる場合はwarm startを使わない。問題名に依存する分岐は追加しない。

有限下限を子でシフトしても列と費用は同じで、RHSと変数幅だけが変わる。非基底の上下限状態を新しい幅へ移し、同じ基底で再計算する。親のdual feasibilityが残れば、primal infeasibleな基底変数だけを修復できる。factorはArcで共有し、新しいeta更新を子に持たせ、stackの両兄弟に同じ親stateを参照させる。

bound-flipping ratio testの候補導出を検証する。違反基底行rに対してw=B^-T e_r、alpha=A^T w。下限違反ならt=+1、上限違反ならt=-1。非基底の移動方向s=+1(lower)/-1(upper)、q=reduced_cost*s。denom=-t*alpha*s>0の候補をq/denom順に扱う。有限幅で違反を修復しきれない候補はflipし、残る違反を修復できる列をpivotする。dual更新はy_new=y-t*beta*w。符号・境界・退化・数値安定性は導出だけを信用せず、一次文献と独立差分テストで確認する。

dual feasibilityや数値残差が不安定なら、残り時間内でcold二段階法へ戻す。候補なしを無条件にinfeasibleと扱わず、元行列と全変数境界に対するFarkas/区間証明を数値検査する。証明できなければcoldへ戻し、誤枝刈りを採用しない。

warm/coldを同じcanonical列構成で切替可能にし、26基本テスト、全列挙/HiGHS小問題1000問、合成根LP200問、開発MIPLIB根LP6問を検証。境界変更の連鎖と兄弟復帰、冗長行、固定変数、自由変数の変換変更、時間切れ、悪条件行列を追加検証する。warm成功/dual pivot/flip/cold fallbackを数え、5秒の既存開発セットで同条件比較して採否を決める。全240問baselineの検証群を改善判断には使わない。

開発の重いコマンドは`taskset -c 32-63`、短期benchmarkはCPU0。稼働baselineのCPU8..15・snapshot・共有.venv/システムsolverライブラリに触れない。
