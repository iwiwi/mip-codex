# 探索順の比較と主解専用探索

## 採用した動作

主探索は従来の深さ優先・floor側優先を維持する。根LPの修復と潜りの後に、推定コスト順の短い探索を主解専用で行う。主探索と同じ自作Rust solveを一段だけ呼び、再帰先のprobeは必ず無効にする。既存のLP/MIP最適化実装への委譲はない。

probeは4096ノード、全体時間の20%または1秒の小さい方、残り全体期限でも制限する。推定コスト分枝・DFSを用い、修復とLP潜りは重複して実行しない。取得した主解を元Modelの境界/整数性/制約と外向き区間で検査し、改善する場合だけ採用する。probeの下界、optimal/infeasible/unbounded等の終了状態は主探索の証明や枝刈りに渡さない。候補解の元MPSへのGMP検査も別途必須。

probeのnodes/lp_solves/seconds/status/max_open_nodes/candidates/improvementsを別記する。Result.nodesとNODES制限は主探索のノードを表し、probeは独立した4096ノード上限を持つ。LP総数/pivot/flip/refactor/warm統計にはprobeを含める。主探索の待ちノード数とprobeの最大待ちノード数は分けて出力する。主探索が根LPの下界までの主解を既に持つ場合はprobeを省く。対象は疎LP版。

MIP_PRIMAL_PROBE=0/1で切替、既定有効。主探索の既定を一律estimateへ変える場合より悪化を抑えるため、別探索の役割を主解発見に限定した。問題名や既知値で分岐しない。

## 任意の待ち行列

MIP_SEARCH=depth/best/hybrid。既定depth。bestは最小の最小化下界を持つ未処理ノードから取る。hybridはDFSを行い、64popごとにDFS待ちノードをheapへ移し、全体の最小下界から再開する。下界が同じなら後に積んだ優先子を先に取る。同じノードを両構造へ複製せず、終了時の全体下界には両方の未処理ノードを含める。主探索の枝刈り条件は変更しない。最良下界だけでは整数解まで到達しにくいケースもあり、現時点で既定には採用しない。

## 固定開発18問での対照

以前選定した開発群だけを使用。8問はCPU0各5秒、残り10問はCPU1各2秒、1thread/16GiB。全モードを同じsource/資源で比較し、生解を元MPS GMP絶対・公式相対で検査した。holdoutを調整に使用しない。主解なし同士は「同等」に含む。

|比較|改善|悪化|同等|
|---|---:|---:|---:|
|child_order/rust-pseudocost-nearest|2|4|12|
|child_order/rust-pseudocost-estimate|4|2|12|
|queue/rust-best|3|6|9|
|queue/rust-hybrid|1|5|12|
|queue/rust-best-estimate|2|7|9|
|queue/rust-hybrid-estimate|4|5|9|
|probe/rust-probe|3|1|14|

子優先順の全54runは時間制限・証明0、返却37解は両検査に合格。estimateはgmu-35-50の主解-112379.8182→-2540189.7008、gmu-35-40 -933357.2996→-1860472.4551へ改善したが、mad/binkarが悪化した。nearestではgmu2問の主解がなくなった。

probe無効の待ち行列5方式×18問は全90run時間制限・証明0、返却54解は両検査に合格。best/downはpk1を17→12、下界0→4.8242、50v-10を3777.60→3559.30、下界2879.07→3019.95へ改善したが、mad/binkar/markshareなどで主解が悪化した。gmu2問はbest/hybridの全4方式で主解なし。単一の方式の全体既定化は棄却し、任意機能として残す。

probe対照167816dは3改善/1悪化/14同等 (主解なし5問を含む)。gmu-35-50 -112379.8182→-2532435.2987、gmu-35-40 -933357.2996→-1860472.4551、gen-ip002 -4703.110424→-4706.617100。pk1は17→19へ悪化。全36run時間制限・証明0、返却26解は両検査合格。gmu2問の大きな不足を減らし、主探索全体をestimateへ変える場合より悪化が少ないことからprobeを既定化する。pk1の悪化を隠さず、全体優位を達成したとは主張しない。

全記録/再集計手順はartifacts/search-summary.jsonとsearch-summarize.py。各source/binary/input hash、設定、exit、時間、解、検査をsmoke各ディレクトリへ保存。既定化後の新規8問×4solver比較は別に記録する。

## 正しさの検査

86 Rustテスト。Queueの取出し順/同順位、各待ちノードを一度だけ保持、stackとheapの双方の最小下界を検査。80二値MILPを整数演算の全列挙で比較し (最小化/最大化各40問)、、3探索を照合。半整数等式の130ノード打切りでも誤ってinfeasibleとせず全未処理ノードの下界を維持する。probe内のinfeasibleを全体の証明へ使わない、根ノード上限1でも主解と根下界を区別する、期限0でprobeを開始しない回帰を追加。

新best/hybridの各1000小問題+200中規模、probe有効の1000+200、既定化後の1000+200をHiGHSと照合し、純整数では全列挙も照合。返却主解は元MPS GMPで検査。いずれも失敗0。従来の修復/分枝/伝播/潜り単独の検査ではprobeを明示無効にし、各機能の保証を混ぜない。

最終f1c8a2fの新規8問×4solver各5秒 (smoke-20261115-071815) は全32run時間制限・証明0、返却28解はGMP絶対27合格/既知HiGHS pk1不合格1、公式相対28合格。Rust7解は双方全合格。比較表はBENCHMARK、全bound/gapはartifacts/search-default-summary.json。probeで主解を得た後のbest/hybrid組合せは未評価であり、次の実験に残す。
