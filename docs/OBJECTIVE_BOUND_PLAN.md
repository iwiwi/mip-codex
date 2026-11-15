# 保証付き下界と整数目的値丸めの実装計画

PR18後の試作。まだ丸めを既定採用しない。LPが返す近似boundを無条件ceilすると微小な過大評価が1単位の誤枝刈りへ拡大するので禁止する。

1. 原MPSの目的係数/offsetを包含する区間を保持。小整数トークンだけから正確に得た値はpoint、その他は一度丸めした値の隣接f64で外囲い。従来JSON/手作りModelはNone。数値自体は変更しない。
2. 原integer変数から開始し、厳密整数係数の等式で唯一の未確定変数の係数が±1なら暗黙整数性を導く。連鎖はrow queueで処理し、各推論の変数/元rowを証明として残す。費用とoffsetが厳密整数、非零費用の変数が全て整数と証明できればgcd費用をstepとする目的latticeを得る。
3. revised::Answer.witnessはoptimal時の行双対乗数を持ち、scalingで元Problemの単位へ戻る。standard_sparseへ原row index/符号のmapを追加する。ranged splitは同じ原rowへ乗数を合算する。任意の有限乗数を用いた下界なので、mapの丸め自体は妥当性を壊さず、良い乗数を得るためのもの。
4. 原LP用Model (証明済みglobal cutを含む) について、任意の行乗数yから
   `sign*offset + sum(y_i * (lower_i if y_i>0 else upper_i)) + min_box sum((sign*c_j - sum(y_i*a_ij))*x_j)`
   を外向き計算する。必要なsideが無い乗数は0へ置く。原係数/辺の包含区間を使う。有限boxは両端の積の最小値、非有界方向に負のreduced costがあれば微小でも証明を棄却する。
5. 初版は全ての有限元変数境界が厳密小整数pointで、伝播無効の場合だけ。これならNode.lower/upperは原境界と実際の整数分枝から得た制約であり、そのまま証明boxに使える。非point境界・dense engine・不明情報は追加丸めをしない。現在の通常float boundと別に保証付きboundを計算し、その値だけを目的latticeへ上げる。
6. intervalが非有界列の符号を確定できず、対象の費用/係数がpointなら既存exact::Sumでreduced costを再計算する。厳密0は0として扱える。必要ならy全体を1±2^-40等へ微小scaleした別の任意乗数も試し、必ず全計算を再検証する。未検証のdual feasible仮定は置かない。
7. certifierは原列の係数区間/行辺を一度だけcacheし、各nodeの点評価を行う。通常の有限列は区間演算、非有界の曖昧な列だけexact累算を使い、全列のBigInt LP化はしない。点0、±1との演算は正確な特殊形で幅を減らす。overflow/非有限は棄却する。
8. `ceil((rigorous_lower - signed_offset)/step)`の引算/除算も下へ丸め、grid値はi128で求め、正確にf64表示できる範囲だけ採用。大きい不明な値は通常動作へ戻す。min/maxの符号を区別する。
9. objective integrality proofを原MPS Fractionで独立検査する。既存verify_cover_cutsのearly returnより先にこの証明も確認し、cut0本・主解なしでも不正証明のbound/optimal/infeasible主張を排除する。稼働native snapshotは不変。

検証: 元十進の包含区間fixture、小数が整数へ丸まった入力の棄却、暗黙整数性の非循環chain、任意乗数の厳密Fraction下界fixture (Rustの下界が常に以下)、負RHS/自由変数/有限box/微小負reduced cost/整数付近の安全な丸め、min/max全列挙、1000小/200中規模、固定18開発問対照。markshareの公式値は検証artifactだけにあり、optimizerへ入れない。

別の次候補は主解なし問題へのfeasibility pump、選択した強分枝子LPのwarm基底だけの再利用。全240最終Rust評価と全体優位は未達成。

## 実装後の判断

全ノード版は60秒markshareで探索増/約8%低下のため任意`all`だけに残す。根限定`1`は18問全主解同等、30n下界2.0297→51、markshare60秒で同一探索を確認した。原境界からbaseと最初の正の増分を導く証明を追加。保証下界に反する許容誤差候補の拒否と、箱の目的最大値による枝刈りも含む。根後にcacheを破棄する。既定化の最終検査中。

既定変更により、整数目的下界をあえて分数のままにする既存cut/diving単体試験が失敗した。各機能の単独試験では目的丸めを明示無効にし、新しい相互作用はobjective_bounds.rsと一般solve試験で検査する。失敗ログも保存する。
