# 開発188問の根LP診断

2026-11-15 23:09 UTC。PR58の凍結source876099b61/FP8262e2efdce05367を使用。Benchmark v2の開発188問だけをmanifestのsplitから選び、holdout52問には触れない。1thread/16GiB RLIMIT、各10秒/1節点、4コア0..3で47問ずつ順番に収集する。全240問の本評価ではなく診断であり、native比較の専用8..15とは分離する。

全cut・主解heuristic・強分枝・丸めを無効にし、create-newの根LP点/行乗数を保存。元MPS hash/frozen config/binary hash、全設定・stdout/stderr・終了状態・wall時間を残す。LP点は整数解と呼ばない。結果が整数主解を含む場合は元MPS検査へ回す。失敗やunsupportedも記録して他の開発問題の収集を続ける。

後でFractionから原制約/境界を検査し、2行集約候補が得られる構造、根LPで時間/数値/メモリに失敗する構造を調べる。nativeの解や最適値を最適化へ注入しない。各partitionはforeground tool sessionで稼働し、親shellが終了状態を保存する。freezeは変更せず、別実装のビルドで走行binaryを入れ替えない。
