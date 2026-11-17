# GLPK比較専用driver

`cc -O2 tools/glpk-benchmark/main.c -lglpk -lm -o target/glpk-benchmark`

最適化は既存GLPKに委譲する比較専用の独立実行ファイル。Rustソルバーから呼ばず、リンクもしない。MIPLIB公式testscript v1.0.4のrun_glpk.shに合わせてcuts/feasibility pumpを有効化し、presolveを含むglp_intopt呼出し全体に時間制限を設定。callbackは探索を変えず、丸められた進捗ログからでは取得できない双対境界を記録する。設定・返却status・API返却コード・許容誤差も保存。

`nodes_created`はGLPKの累積生成ノード数。処理済ノード数と同一視しない。根LP中の停止など、callbackが有効な境界を返していない場合はdual_bound=null。

このdriverはGLPKにリンクするためGPL-3.0-or-laterで配布する (LICENSE)。他のRustソルバー本体から独立している。GLPK 5.0のソース: https://ftp.gnu.org/gnu/glpk/glpk-5.0.tar.gz

境界観測は最大1回/秒に制限する。`glp_ios_best_node`がactive node一覧を走査するため、毎callbackで呼んで探索速度を損なうことを防ぐ。最後の正確なcallback値と経過時間を保存し、終了時の未知のより強い境界を捏造しない。最適終了時はdual=primalにする。

GLPK MPS拡張の目的定数RHS符号を、CPLEX/HiGHS/Rust readerの負RHS規約に正規化する。変更前後の定数を保存。公式Benchmark v2全240問の定数は0なので、当該集合の探索を変えない。GLPKがOBJSENSE等の未対応入力を拒否した場合は、その失敗もそのまま報告する。
