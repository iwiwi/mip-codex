# 本体cover選択の構造依存heap候補

容量候補1572/ABCから別枝。private heap8d0826のStateだけを移植し、MIP_GRAPH_COVER_HEAP既定OFFで比較する。認識・探索budget・反復上限・再始動・乱数・境界計算はそのまま。旧APIはscanを維持するwrapper、新APIだけheap選択を許可。

private fixed100k×4モデル×3pairs全24点は同values/同weighted counters/GMP valid。疎pair16x、z26約3.6x、S7約1.13x、密cod105は5x遅い。一律適用を棄却し、n>4096かつ8Eceil(log2 n)<=n²の構造判定だけでheapを選ぶ。Eは既存構築済み辺数、問題名・既知値では選択しない。追加16nバイト分を既存128n見積りに加え144n+32E<=memoryも必要。満たさなければscan、heap配列を確保しない。

score降順・age昇順・頂点番号昇順で線形走査と同じ選択。移動時にcovered頂点とcovered隣接点を更新、未被覆辺重み変更は両端非covered、forget時は再構築。Stats.heap_usedで実適用を報告。unit testsで強制heapを約60000状態の全走査と照合し、頂点4999の辺/heap位置、gateのサイズ/密度/メモリ制約も検査する。

本体の生成例・厳密証明・元MPS GMP、z26固定60秒、開発188問の全376検査を行って採否を判断。private60秒3ペアは自作共通seedからscan−1190/heap−1193、全GMP validだが、本体成績や新記録ではない。最強OSS/新記録目標は未達。
