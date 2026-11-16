# ノード間基底再利用

2026-11-15実装。親の最適LP基底・非基底の上下限状態・LU factor/eta列を共有し、子の境界変更後に基本解を再計算してbounded dual simplexで修復する。`MIP_LP_WARM_START=1`で有効、`0`でcoldと比較する。既定は有効。coldへの切替も引き続き可能。

当初は人工列のcanonical化を考えたが、親の拡大行列自体を保持すれば番号は変化しない。これによりcoldの二段階法を変更せず、不要な正負人工列の追加も避けられる。元の列/費用/等式フラグ/slack幅をハッシュでなく完全比較し、自由変数の変換などで構造が変わった場合はcoldへ戻す。問題名に依存する処理はない。

有限下限のシフトでは列と費用は変わらず、RHSと変数幅だけが変わる。非基底の上下限状態を新しい幅へ移し、親のLU/etaで再計算する。双対実行可能性が維持されていれば、違反基底変数を修復するだけで再最適化できる。factorとeta列はArcで共有し、各ノードは自分の基底番号・上下限状態・更新列だけを持つ。兄弟による親状態の変更は起こさない。

違反行rに対してw=B^-T e_r、alpha=A^T w。下限違反ならt=+1、上限違反ならt=-1。非基底の移動方向s=+1(lower)/-1(upper)、q=reduced_cost*s。denom=-t*alpha*s>0の候補をq/denom順に処理する。有限幅で違反を修復しきれない候補はまとめてflipし、修復できる列をpivotする。符号関係はy_new=y-t*beta*wに対応する。

全解で元の拡大行列に対する主・双対残差と目的値一致を検査する。候補なしをそのままinfeasibleとせず、行乗数と全box境界の区間がRHSと矛盾するか検査する。数値的に検査できなければcold二段階法へ戻す。dual feasibilityが失われた場合も同様。時間/反復上限に達した時はcoldを再起動しない。

dual退化はprimal thetaではなくdual betaで検出し、ゼロdual stepではBlandの行/列選択を使う。warm pivotがmax(64,4*m)を超える場合もcoldへ戻し、cache利用が停滞し続けることを避ける。

最適終了時のdual側では毎ノードのLU再分解を必須とせず、50 eta更新または残差異常時に再分解する。残差の独立検査は毎回行う。warm無効時は使わないcacheを生成せず、比較の余分なコストを避ける。

検証: 基本35テスト、上下限/RHSの3000変更をcoldと差分比較、HiGHS/全列挙の小規模1000問、追加中規模200問、合成根LP200問、開発MIPLIB根LP6問。cold fallback、兄弟復帰、ranged slack、人工列、固定変数再活性化、構造変更拒否、期限切れを含む。性能比較は開発用のみで行い、全240問baselineの検証群を改善判断に使わない。

参考数学 (実装コードの移植なし):
- Huangfu, High performance simplex solver, §3.2: https://www.maths.ed.ac.uk/sites/default/files/atoms/files/huangfu.pdf
- Koberstein, The dual simplex method, techniques for a fast and stable implementation, §3.2: https://webdoc.sub.gwdg.de/ebook/dissts/Paderborn/Koberstein2005.pdf

Harris比率検定、コスト摂動、dual steepest edge、hyper-sparse価格付けは未実装。開発の重いコマンドはCPU32..63、短期benchmarkはCPU0。稼働baselineのCPU8..15・snapshot・共有.venv/システムsolverライブラリに触れない。

採用判断: 初回の退化誤判定版はmad/markshare_4_0で遅くなったため採用しなかった。修正版の5秒ablationはpk1 29→20、mad 0.4938→0.2576、markshare_4_0 4→約2に改善。大きめ3問のうち50v-10/binkar10_1はwarmのみprimalを得た。30n20b8ではprimal/dual品質は改善せず、探索ノード数も減った。この結果も省略せず保存する。最終21runの返却17解はPython/GMPの双方に合格。全体の性能優位は未達成。
