# 数値失敗時だけの基底残差補正

2026-11-15。fresh LUのratio失敗時に、既存exact::dotでr=b-Axを作り、同じ汎用LUでB*d=rを解いて基本変数のみ補正する。最大3回、期限確認あり。基底境界が既存FEAS内になり、元ratioも通り、max |r|/(1+|b|+|A||x|)が悪化しない点だけ採用する。失敗時は点を変更しない。許容値/親basis/列/目的を変更しない。文献と診断はRATIO_RESIDUAL_DIAGNOSTIC.md。

初期glass4比較では局所LP2件を回復したが、最終主解3466706233.3333354/163節点/numerical_errorは不変。app1-1も59節点で不変。元sourceは診断branch299db042にproduction補正を加えたもの、ログbasis-refinement-pairsに保存。次は最終certificateの基底境界失敗にも同じ補正を試す。更新etaが残っていれば先に再分解し、補正後も元certificate/元単位/元MPSの検証を全て維持する。
