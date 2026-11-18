4096頂点を超える厳密なuniform-objective binary packingも、自作グラフ探索で処理できるようにします。元の整数係数・0/1領域・非binding行の判定、10Mペア作業、メモリ、期限のguardを保ち、頂点上限をu16に収まる65535へ拡張します。既存public APIは4096のまま、新APIとCLI `MIP_GRAPH_LARGE_CAPACITY=0/1` で比較できます。最終候補は既定ONです。

大規模グラフだけ再帰深さ512で `depth_limit` を返し、安全な根/prefix境界と検査済み候補解を保持します。単一5-cycle＋孤立点は2節点で正しくoptimal、多数の5-cycleは512節点で安全に中断することを元MPSから検査しました。

- 最終core1572f4382c / FPabc0956a38a27666: 350 Rust成功・4ignored、200小規模/1200出力の全列挙・GMP・厳密証明、7410改変棄却。4097–5484頂点40解析例も全GMP/証明/最適値一致、160改変棄却。深さ制御4点GMP/証明と2改変棄却。
- 初期core580d517295 / FPf293: 開発188問でprimal187同じ/1改善/0悪化、dual173同じ/1改善/1悪化/13比較不可。全376出力を新規proof検査、128点GMP＋公式検査。大容量の新認識0件なので、その改善を本機能へ帰属しません。fast0507のdual悪化も保存。既定ON最終ソースの188問/376検査は進行中です。
- 240問外z26、60秒・1thread・16GiB、本体OFF: −7/LB−1651、最終ON: −1154/LB−1578。全主制御7点は元GMP・独立root/prefix/Fraction証明valid。初期FPf293のLB−1575とは区別します。
- 同機・同資源の新規native60秒: HiGHS1.16.0 −1147/LB−2114、SCIP10.0.2 −1013/LB−1650.9375（2点GMP valid）、GLPK5.0は解/境界なし、60.903269秒の超過も保持。外部解を自作探索には渡していません。この1例から全体優位や新記録を主張しません。

初回full testのENOSPCと、それに伴うbaseline管理停止は失敗ログを保存し復旧済みです。既存719件のSHAと稼働GLPK case identityを維持して凍結coordinatorのみ再開し、最適化の再実行はありません。残るnative1件は実行中。holdout52の個別成績は参照していません。自作final240は未起動です。

初期全評価は `artifacts/large-capacity-evaluation`（7222ファイル、全payload SHA照合）、私的原型は別archive、前PR126のprimary再検査と未起動snapshot302bも保存。採否・最終再測定の完了後にreadyへ変更します。

その時点のAI資源量（ローカル各session最終累積を一度だけ集計）:
```json
{
  "sessions": 1,
  "totals": {
    "input_tokens": 1784126284,
    "cached_input_tokens": 1771508608,
    "output_tokens": 8807328,
    "reasoning_output_tokens": 5399411,
    "total_tokens": 1792933612
  },
  "note": "input_tokensはキャッシュ分を含む。reasoningはoutputの内数。ローカルログの最終記録まで。"
}
```
