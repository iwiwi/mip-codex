# 評価240問外のpacking入力による汎用探索調査

cdc7-4-3-2はmanifest全240問に含まれない。公式元MPSのみ取得し、gzipおよび展開後SHAを保存。既知解ファイルは取得せず、掲載bestは報告だけに用い、探索の入力/停止値に与えない。公式詳細 https://miplib.zib.de/instance_details_cdc7-4-3-2.html は2026-11-18確認時open、11811binary/14478rows、best−333（2026-08-28掲載）。

main128/9c9の元点から60秒、CPU46/1thread/16GiB、HiGHS1.16.0/SCIP10.0.2/GLPK5と順次比較。研究driverはprocess group単位の120秒watchdogを使い、終了code/実時間/全stdout/stderr/API結果を保持。比較点・native情報を自作探索へ一切渡さない。

| solver | primal | dual | seconds | point GMP |
|---|---:|---:|---:|---|
| main128 | −279 | −424 | 60.004395 | valid |
| HiGHS1.16.0 | −280 | −1111 | 61.059365 | valid |
| SCIP10.0.2 | −237 | −11744 | 60.346490 | valid |
| GLPK5 | none | none | 60.473351 | no point |

mainのstrict bound proofもvalid。時間超過もそのまま保持し、GLPKの点なしを誤って実行不能とは扱わない。主解はHiGHSに1点劣り、下界はmainが強い。新記録・最適性・全体優位の主張なし。

private既存heap componentはCPU40、元点からsingle/reseed各60秒が両−285、2点GMP valid。これは本体と異なる時間配分でありmain成績へ帰属しない。自作285seedから300秒single継続は−290、300.002544104秒、元GMP valid。掲載−333には43点届かず、新記録なし。新main候補では既存の大規模疎heap gateが適合する構造だけ局所探索時間を5%→25%にする。すべて汎用処理で、問題名による分岐は加えない。
