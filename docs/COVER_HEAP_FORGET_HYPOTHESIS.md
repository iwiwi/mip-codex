# 私的mean-weight忘却閾値の上限

同順位heapの速度検証とは別の探索方策仮説。大きなグラフで旧平均重み閾値n/2が高くなり長く忘却されないため、2total>=E*min(n,128)、すなわち平均64以下で既存3/10縮小を行う私的variantを作る。問題名・目的値・停止値は使わない。private元heap source8d0826をコピーし、この条件だけを変更、main srcはE5不変。選択軌跡一致はこのvariantには主張しない。

全score/未被覆辺/heap順位を再計算する既存6testsに、5000pairで全点非covered状態から80回のweight増加を行い実際のforgetting後を検査する1testを加える。例示test/buildだけで通常CLIを変えない。凍結後は原型と同じ自作seedから固定300秒で比較、全点元MPS GMP。外部解・既知目標は与えず、新記録/本体成績は検査・本体再現まで主張しない。

結果: source71c23b0a46のexample7tests成功。5000pairで実際のforgetting後にも全score/heapを再計算照合。共通自作1197seed/CPU45/300秒では旧thresholdが−1197・57,065,252反復・forget0、新mean64が−1197・35,503,439反復・forget402。両元MPS GMP valid。原型より品質が上がらず観測throughputも下がったため、本体移植は棄却。source4SHA・凍結ELF・全出力を保存。
