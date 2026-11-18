# 頂点penaltyによる異なる局所探索の次仮説

自作1198からedge-weighted探索を3本各1800秒・計10億超の反復まで続けても改善0だった。重み付けの異なる探索を調べる。

参考は Pullan & Hoos, Dynamic Local Search for the Maximum Clique Problem, JAIR25 (2006), DOI10.1613/jair.1815 / https://arxiv.org/abs/1109.5717 。論文の着想（拡張、1頂点交換、頂点penaltyと忘却、摂動）を参考に独立Rustで実装し、他者の実装コードは取得・複製しない。論文のtarget達成停止は使わず時間/反復guardだけを使う。

まだ未実装。元packing認識の無改変コピーを別profileへ分離し、元graph/点の検査を保持する。増分conflict数と候補bucketを小規模無作為graph/全列挙上界・各状態の再計算で検査した後、pd5/10/20を同条件で試す。入力点は自作Rust1198だけでnative/掲載解は使わない。全出力を元GMP/公式で検査し、原問題bound/最適性やmain達成は主張しない。利益があればmainの汎用portfolioへ別途統合・再測定する。

実装後5テスト成功。初回2テストは保持頂点のpenaltyが11を超えるケースで停止し初回source/logを別保存、u64と最大cycle数(maxSteps+1)へ訂正し、singleton回帰を追加した。200小graph×4pdの独立全列挙上界照合と100graphの増分状態再計算を含む。production src/Cargo/buildは無変更、example-only build前後の通常CLI cec43 SHA不変。cod105/生成5000pairs/own1198の3点fresh GMP公式valid。長比較は論文で別の摂動となるpd1も追加し、1/5/10/20の全4値を長結果を見る前に登録。各同じ自作1198、1800秒guard/100Msteps上限/1920group watchdog、CPU40/47/50/51、16GiB/1thread。新記録やmain達成はまだない。
