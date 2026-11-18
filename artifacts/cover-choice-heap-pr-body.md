大規模で疎なpackingのweighted-cover探索では、毎反復の全頂点走査が律速になります。同じscore降順・age昇順・頂点番号昇順を保つindexed heapへ替え、固定探索時間で多くの反復を実行します。

適用はn>4096かつ8Eceil(log2 n)<=n²、追加メモリを含む144n+32E<=memory。外れる場合はscanとしheap配列を確保しません。CLI `MIP_GRAPH_COVER_HEAP=0/1`、最終候補の既定はON、旧public APIはscanのwrapperを保持。認識・予算・反復上限・再始動・乱数・境界計算は変えません。Stats.heap_usedで実適用を報告します。

- 初期a45/BDDと既定ONa6/9c9を別凍結。main127へはソース/tests/scripts/Cargo/build全byte同一でrebase。各353 Rust成功/4ignored、各1200小規模出力の全列挙/GMP/strict proof、7410改変棄却。各12大規模random-core出力もcomponent全列挙/GMP/proof（実heap6）。約60000更新でheap選択と全走査、全heap位置/順序/scoreを直接照合。
- 私的固定100k反復×4モデル×3pairsは全24点GMP、同values・同weighted counters。疎pair約16x、z26約3.6x、S7約1.13x、密cod105では約5x遅化を観測し、一律適用は棄却しました。
- 最終本体z26/300秒・同CPU44・16GiB1threadはOFF−1163/LB−1575→ON−1182/LB−1578。60秒では両primal−1154、dual−1575/−1578。改善するprimalと変動するdualを両方保存、9主制御点は全GMP/strict proof。初期300秒は−1160/−1578→−1178/−1575と区別します。
- 最終188問10秒はprimal187同じ/1悪化、dual173同じ/2改善/13比較不可、両182time/2optimal/4numeric、finite175ずつ。全376fresh proof/128点GMP＋公式valid。実heap適用0のため改善は本機能へ帰属せず、neos-911970悪化も保持。同一9c9/CPU46の3交互順pairs6点は全両140.59/LB23.26、全GMP/proof valid。初期188はprimal186同じ/1改善/1悪化でした。

300秒のnative同条件比較をCPU46で新規実行中。HiGHS1.16.0は−1147/LB−2114/300.048725秒、GMP valid。SCIP/GLPKを待ってreadyへ変更します。外部解を自作探索に渡していません。private1197点は本体成績や新記録として扱わず、全体優位は未達です。

初期6666ファイル/4bundle、最終6715ファイル/4bundleを全payload SHA照合し、凍結ELFは別保持。private原型・不改善実験、親PR127のprimary再検査/未起動snapshot2bbも別archiveへ保存。正式Rust240は未起動、元native720を保持し外部disk障害重複1件の事前固定再測定が進行中です。holdout個別成績は開発に使用していません。

AI資源量（各session最終累積、一度だけ集計）:
```json
{
  "sessions": 1,
  "totals": {
    "input_tokens": 1805228901,
    "cached_input_tokens": 1792304128,
    "output_tokens": 8986805,
    "reasoning_output_tokens": 5524909,
    "total_tokens": 1814215706
  },
  "note": "input_tokensはキャッシュ分を含む。reasoningはoutputの内数。ローカルログの最終記録まで。"
}
```
