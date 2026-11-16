# 長いPhase Iだけの価格付け再開

2026-11-16。phase-one-dantzig候補b076の188対照で複数主解改善はあるが、60秒Neo860300の3966→11409、eil/bppc/nuの悪化が残った。初期LP点と局所探索経路の変化を広く入れない代案として、Phase Iの最初は従来どおり、既存の正規化pricing budgetと同じmax(8*m,100) pivot+flipを消費した後だけ、退化時の価格付けを再開する。問題名には依存しない。MIP_PHASE1_DELAYED_PRICING=1限定。keep-pricing=0なら従来、keep=1/delayed=0なら即時継続候補と比較できる。

Phase II、許容誤差、残りpivot/deadlineは変えない。初めに3条件の根LPでPhase I消費量と完了時点を確認し、改善がなければ採用しない。既存LP点・元問題証明の検査は維持。mainはPR77 b0b4e946d/FP5a6のまま、全240最終評価は未開始。
