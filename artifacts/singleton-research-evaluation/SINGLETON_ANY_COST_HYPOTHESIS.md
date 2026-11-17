# 非零費用singletonも初期基底に使う案 — 棄却

2026-11-17。親は零費用singleton crash。Phase Iの人工目的では元の費用は使わないため、元費用が非零でも同じ実行可能対角基底の初期化に使える、という仮説。`MIP_LP_SINGLETON_ANY_COST`既定false、CRASH有効の時だけ効く。2の冪・容量・実際の上下限・残差の一致条件、Phase IIの元費用復元、点/双対/不能検証は維持する。直接bool指定の試験で非零の正負費用を検査した。

FPd02feb511068c74e、全273Rust成功4無視。選択6問10秒では改善0。timtab1は1072838→1082993へ悪化（下界は同等）、icir97_tensionは同じ下界でpivot24882→29326へ増加。30n/rococo/nu25は同品質、bab6は両方まだ有限下界なし。採用列は増えたが性能信号がないので188問やnative追加評価へは進まず棄却。12出力の独立監査を開始。主作業木artifacts/singleton-any-cost-v1-*へソース・ELF・測定・失敗を保持する。mainへ追加しない。
