# 主解専用のRINS近傍

既定有効、`MIP_RINS=0`で無効化。根LPと現在のincumbentが一致する整数変数だけを固定し、残りを自作Rust B&Bで解く。新しく2変数以上かつ可変整数の10%以上が1e-6以内で一致する場合だけ試す。元incumbentをseedとして含む近傍を作り、seedと返却点を元モデルの1e-7/外向き区間で照合する。

根で一度だけ、全体時間10%または0.5秒の小さい方。局所1024node/各LP4096pivot。nearest/DFS/pseudocostで探索。局所RINS/probe/diving/repair/strong/cut/伝播は全部無効にして再帰一段に限定する。主探索のノード数と局所ノード数を分け、LP/pivot/warmは総数へ加算する。元Modelは変えず、近傍のコピー内で整数境界だけを固定する。

局所のoptimal/infeasibleや下界は本体へ渡さない。返却点だけを原モデルで再検査し、真に良い目的値ならincumbentを更新する。根LPの下界は変わらず、主探索の通常のLP判定で証明する。private solve_seededへ初期解を渡せるよう分離したが、通常solveはseedなしのまま。予算0のseedが主解であってもglobal optimalと扱わない回帰を追加した。

数学的原理の参考: Danna, Rothberg, Le Pape (2005), [Exploring relaxation induced neighborhoods to improve MIP solutions](https://doi.org/10.1007/s10107-004-0518-7)、[IBM公式RINS説明](https://www.ibm.com/docs/en/icos/22.1.2?topic=heuristics-relaxation-induced-neighborhood-search-rins-heuristic)。外部実装の移植や最適化委譲は行わない。

## 対照実験

固定18開発問、core8問各5秒CPU0/extra10問各2秒CPU1、1thread/16GiB。

|試作|最終主解改善/悪化/同等|判断|
|---|---|---|
|根のみ5%/最大.25秒 (7ae5eb6、091911/091918)|2/0/16|gmu両方に改善、時間切れ|
|根でseedがなければ後のnodeで一度試す (fbe8305、092238/092244)|2/0/16|18問全値同じ、追加LPだけ増えるため棄却|
|根のみ10%/最大.5秒 (c6b4657、092558/092559)|3/0/15|gmu50とbppがさらに良く、既定採用|

各対照36runは全時間制限/証明0。各28返却解はGMP絶対/公式相対全合格、全134cutも元MPS Fraction有効。内部のimprovementsカウンタが立っても最終主解が対照と同じなら、品質改善件数へ数えない。

最終候補の主解:

|問題|off|on|
|---|---:|---:|
|gmu-35-50|-2532435.2987|-2574311.3903|
|gmu-35-40|-1860472.4551|-2367540.5084|
|bppc4-08|72|69|

残る15問は同等 (主解なし4問を含む)。後回し試作はtimtab1 node738/35局所LPで近傍optimalだが改善なし、pg5_34 node52/1局所LPが4096pivotのiteration_limitでseedだけを保持。これらを元問題の証明や改善として扱わない。

118 Rust回帰が成功。小1000/二値1000/中規模200も参照解に一致。元の近傍がseedを含むこと、元Modelを変更しないこと、無効seed/不足一致の棄却、局所optimalでも本体node_limitと下界を保持することを確認。既定化時の試験コードの構文失敗と修正後全テスト、予算置換の誤編集を測定前に訂正したログ、全試作を保存する。

60秒対照を同じ18問で開始。c6b4657のdetached worktreeにバイナリと両checkerをコピーしてhashを固定。CPU4..7の4群で同じ問題/CPUのoff/onを比べ、native baseline CPU8..15や開発CPU0/1と重ねない。結果は3主解改善/0悪化/15同等、30返却解と全134cutは双方/元MPS Fraction検査に合格。gmu両方の改善を保ち、bppは72→65。markshareは両設定が約32秒でoptimal1となり公式値に一致。比較3solverの60秒枠ではHiGHS/SCIPが主解1・下界0で時間切れ、GLPK主解5で時間切れだった。単一の開発例で全体の優位を主張しない。frozen manifestはartifacts/rins-60s-frozen.json、実ジョブと終了状態はgroup1..4-launch.logおよびworktree内のsmoke群。全4群と追加native比較はexit0、結果を主レポジトリへ回収・独立再監査済み。
