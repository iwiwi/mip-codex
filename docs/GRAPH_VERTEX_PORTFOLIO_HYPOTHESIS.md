# 大規模疎packingの主解探索を二方式で組む仮説

自作Rustによる段階診断は、同じweighted初段15秒の点から継続15秒で、z26がweighted/vertexとも−1184、CDCがweighted−284/vertex−290となり全6点GMP・公式validだった。入力は両方とも正式240問の外。これは全solver同時間成績ではないため、本体の同時間比較を行う。

main131+未採用long既定ON source4d4b1b5e3aを基に別branchを作る。MIP_GRAPH_VERTEX_PORTFOLIOの既定はOFF。local/cover/adaptive/portfolio/heapが有効で、既存heapの大規模疎判定に適合し、64bitかつ512n+32E<=memory/2の場合だけ新方式を許可する。古い公開APIは新flagfalseのwrapperを保ち、Limits経路のみ新しいsolve_with_vertexへ接続する。

既存の全体期限・局所配分（条件適合時25%）は変えない。初期improve_seed後の残り局所時間を半分weighted、残りvertex pd1へ割り当てる。weightedが早く終われば残時間をvertexへ使う。旧weightedの全時間と比べ品質が落ちる可能性も、同時間で測定して保存する。graph認識・上下界・枝刈り・許容差は変更しない。新点はGraph上の独立集合を厳密検査し、不合格なら元点へ戻す。

新方式は独立実装済みのprivate vertex-penalty profileを移し、モデル読込みではなく認識済みGraphを使う。参考: Pullan & Hoos, JAIR25(2006), doi:10.1613/jair.1815, https://arxiv.org/abs/1109.5717 。他者の実装コードは取得・複製せず、外部最適化へ委譲しない。pd1固定、汎用固定RNG、上限100Msteps/maxSteps+1cycles、target/既知点/問題名による停止なし。

conflict数<=65534、steps<=100M、cycles/penalty増分<=100M+1なので64bit計数は安全。隣接vector容量を32E+40n、2候補bucketと位置等を368n、selected/best/記録等を追加しても512n+32Eを使う保守的data見積り内とする。Graph本体は既存の割当枠を維持し、weighted状態破棄後にvertex状態を作る。

旧privateの5テスト（200小graph×4pdの全列挙上界、増分状態、singleton/cycle guard等）を継承し、memory gateとGraphの補集合隣接生成（63/64/65/127/128/129境界も含む）を追加する。初回はtestの整数型推論でcompile停止、logを保存しusize明示で修正した。現在関連7テスト成功、全367検査とbuildを実行中。

次は同一凍結CLIでvertex0/1をz26/CDC各60秒、large random12出力/独立core列挙、小型1200出力/7410改ざん拒否で検査。全点元MPS GMP/公式、全proofを保存する。利益があれば188問・300秒対照へ進む。長roundの採用判定とは混ぜず、main132統合後はRust/Cargo/buildの一致を確認して履歴を合わせる。holdout52は調整に使わない。数学的最適性/新記録/main達成はまだない。

初期622b629e78/FP a7f019a617fb33e8、CLId31034d868a411814d5db50f072182c28ad0abd97056edf35a9f8ad6c2059869を凍結。全367 Rust/4ignored、1200小出力と7410改ざん拒否、12大規模出力GMP/proof/独立core boundが成功。大規模vertex実行6/heap12/long12。9controlsはCPU40、現在実行中。sourceをこれ以上変更せず、初期整数型compile失敗もlog保存。
