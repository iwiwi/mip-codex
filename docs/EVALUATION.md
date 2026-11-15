# 評価仕様 v0（実装前の事前登録）
公式 https://miplib.zib.de/download.html のBenchmark Set v2全240問。圧縮・展開MPSのSHA256、リスト、URLを保存。既知値は評価専用。名前のSHA256順で先頭160問を開発、残り80問を検証。検証は改善選別に常用しない。最終評価は240問全件。

主評価: 各実行1物理コア、壁時計3600秒、アドレス空間16 GiB、seed 0、相対gap停止1e-6。同じCPU集合・資源制限。外部監視は猶予5秒後強制終了。同時実行最大16、コア重複なし。開発用1/10/60秒は主評価と混ぜない。

解決数、独立検査済み実行可能解数、primal/dual bound、gap、時間、ノード、RSS、終了コードを保存。timeout/OOM/crash/unsupported/invalid_solution/numerical_failureを全件報告。
gap=|p-d|/max(1,|p|,|d|)。未解決を3600秒とした時間のシフト幾何平均（shift=10秒）、PAR-2、共通解決部分の時間を併記。失敗を高速解決に数えない。
独立検査器は元MPSの境界・制約・整数性・目的値を検査し許容差・最大違反を保存。解検査のみを最適性証明と呼ばない。証明状態は小問全列挙・外部比較、将来の証明ログで補強。

SCIP・HiGHS・GLPKの導入可能な安定版を次の環境固定コミットで確定。版、ソースcommit、ビルドフラグ、設定全文、バイナリハッシュ保存。原則デフォルト、1スレッド、同じ時間・メモリ・gap。変更時は理由と再実行を記録。

環境: aarch64 Neoverse-V2 64コア、247 GiB RAM。
根拠: https://miplib.zib.de/set_benchmark.html 、 https://miplib.zib.de/CHANGELOG.html 、 https://webdoc.sub.gwdg.de/ebook/serien/ah/ZIB/ZR-12-05.pdf
性能優越は測定前に主張しない。
