# 分枝・探索の次段階

2026-11-15。現行は方向別擬似コストで変数を選び、floor側を先に深さ優先で探索する。最もfractionalな変数を選ぶ従来方式とnearest/estimate探索順も実験用に残す。主解修復は既定有効。LPが速くても、初期主解/下界の進展と探索木の大きさが律速になる。

観測した子LPの目的増分から方向別擬似コストを学習する方式を実装・検証して採用した。参考: [SCIP公式パラメータ](https://scipopt.org/doc/html/PARAMETERS.php) と [SCIP Optimization Suite 8.0](https://or.rwth-aachen.de/files/research/repORt/scipopt-80.pdf)。前者は方向コストの信頼度やstrong branching予算を設定し、後者は信頼度の低い候補をstrong branchingで評価する方式を説明する。実装を移植せず、Rustで独自に実装する。

実装は順位付けだけを変更する。各子ノードに分枝変数、方向、分枝距離、親LP目的を保存し、実際に解けた子LPの増分/距離を方向別平均へ加える。サンプル不足時は集計平均を補助にし、未観測/全ゼロなら従来のfractional指標へ戻す。ヒューリスティックの目的や不確かな値を下界として使わない。infeasible子の扱いも順位の補助に限り、無限コストを証明に混ぜない。

別に、整数に近い側を先に潜る探索順、両方向の予測増分が小さい側を先にする探索順を対照実験する。最終primalだけでなく初回主解時間、下界、gap、ノード、LP回数と数値異常を保存。分枝変更でLPの退化経路が変わるため、1000小問題/200中規模と全列挙/独立有理数検査を必ず通す。

この後で、低信頼度候補だけを予算内strong branchingで初期化し、計算済子LPのcacheを再利用する。best-bound/hybrid探索はfrontierメモリと親由来の弱いboundの扱いを先に設計する。問題名・既知値・holdout結果による調整は禁止。

先にgmu-35-50/35-40の根LP numerical_errorを診断する。全方式共通の失敗であり、HiGHS根LPはoptimal。初期残差、LU/eta、最小比率検定、phase-I certificateを分けて記録し、許容誤差を緩めて成功扱いにしない。
