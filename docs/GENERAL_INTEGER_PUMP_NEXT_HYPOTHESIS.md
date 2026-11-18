# 次の主解探索候補：一般整数の距離LP

2026-11-18、まだ実装・起動していない仮説。Farkas投影でcrnaの早期数値停止は解消したが、300秒82630nodesでもprimalなしだった。保存Resultのpump.calls=0、divingは10LP・候補0、probeは25nodes/1秒・候補0。ソースbinary_pump_with_global_deadlineは非固定整数変数の元boxが[0,1]に含まれないと、stats.callsを増やす前に返る。

HiGHSのreaderだけで開発4入力のdomainを確認した（最適化なし）。crnaは524変数すべて整数、378個が非binary（例[0,11]、[0,4]）。nidda/glass4/leo1の非binary整数は0。名前をsolverの分岐条件には使わない。domainとpump非作動が次の汎用能力不足の根拠である。

候補は、LP点を丸めた一般整数target tに対し、d>=x-t、d>=t-x、d>=0を補助LPへ加えてΣdをRustの既存LPで最小化すること。標準形の元変数写像を使って補助行を構成し、既存binaryのみの経路は保つ。距離LPの境界・最適性は大域探索へ使わず、写像・整数丸め・元行の厳しい主解検査に合格した点だけ返す。cycle perturbationは固定generic RNG、±1と元node boundsを使い、問題名・既知解・目標値を渡さない。メモリgate・同一global deadline・根pumpの局所時間予算を守る。

まず文献のprimary sourceを確認して独立実装し、既定OFF。符号/offset/固定変数/非binary整数/continuous混合/期限切れ/補助dual非昇格を小問題の独立全列挙・GMPで検査する。Farkas投影の既定ON候補が採否確定してから、その同一基準上で小さい開発pilotを行い、点が増えるか・代わりに下界進行が損なわれるかを両方測る。未実装のため成果・新記録・主ソルバー改善とは数えない。
