# 頂点penaltyによる異なる局所探索の次仮説

自作1198からedge-weighted探索を3本各1800秒・計10億超の反復まで続けても改善0だった。重み付けの異なる探索を調べる。

参考は Pullan & Hoos, Dynamic Local Search for the Maximum Clique Problem, JAIR25 (2006), DOI10.1613/jair.1815 / https://arxiv.org/abs/1109.5717 。論文の着想（拡張、1頂点交換、頂点penaltyと忘却、摂動）を参考に独立Rustで実装し、他者の実装コードは取得・複製しない。論文のtarget達成停止は使わず時間/反復guardだけを使う。

まだ未実装。元packing認識の無改変コピーを別profileへ分離し、元graph/点の検査を保持する。増分conflict数と候補bucketを小規模無作為graph/全列挙上界・各状態の再計算で検査した後、pd5/10/20を同条件で試す。入力点は自作Rust1198だけでnative/掲載解は使わない。全出力を元GMP/公式で検査し、原問題bound/最適性やmain達成は主張しない。利益があればmainの汎用portfolioへ別途統合・再測定する。

実装後5テスト成功。初回2テストは保持頂点のpenaltyが11を超えるケースで停止し初回source/logを別保存、u64と最大cycle数(maxSteps+1)へ訂正し、singleton回帰を追加した。200小graph×4pdの独立全列挙上界照合と100graphの増分状態再計算を含む。production src/Cargo/buildは無変更、example-only build前後の通常CLI cec43 SHA不変。cod105/生成5000pairs/own1198の3点fresh GMP公式valid。長比較は論文で別の摂動となるpd1も追加し、1/5/10/20の全4値を長結果を見る前に登録。各同じ自作1198、1800秒guard/100Msteps上限/1920group watchdog、CPU40/47/50/51、16GiB/1thread。新記録やmain達成はまだない。

mainへの将来統合には初期点なしでの品質も必要なので、同じ凍結ELFでseedなし60秒/100Mstepの4設定もCPU43/45/49/52で測定する。登録時点でseeded pd1は100Mstepsで1198不改善、他3seeded結果は未完了。新しい入力点や目標値は与えない。全4出力GMP/公式検査を行い、seeded測定とは別の品質として報告する。

4設定の100Mstep seededは全1198のまま、101.671223768/140.693012472/149.082632761/175.060531017秒でstep_limit。seedなし60秒はpd1/5/10/20で1186/1126/1130/1129。全8点fresh GMP公式valid、seeded親4点fresh GMP、改善/新記録なし。pd1はseedなしで比較的良いが、mainと同時間の統合比較は未実施のため採用しない。全結果と停止理由を保持する。

次は初段weighted15秒（初期点なし）から得た同じ自作点をweighted15秒とvertex pd1 15秒へ分岐する段階診断。z26/CDCの両方で比較し、各branchの最適化総時間は初段+継続の30秒、全solver同時間成績とは主張しない。CPU40/47、16GiB/1thread、全6点GMP公式、固定100Mstep cap/targetなし。点は全て自作Rustで、外国solver点は使わず、利益と損失を全保持してから本体への適否を判断する。

段階診断全6点fresh GMP公式valid。z26は初段−1181からweighted/vertexとも−1184、CDCは初段−279からweighted−284/vertex−290。元の同じ初段点を使い双方継続15秒、全solver同時間成績ではない。これを根拠に別branch graph-vertex-portfolioで同じmain主解枠を二分する候補を実装し、OFF/ONの全solver対照へ進む。private既存sourceは無変更。
