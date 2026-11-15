# MIPLIB 2017公式solution checker

検証専用の独立プログラム。Rustソルバーから呼ばず、最適化をしない。元MPSを独自readerで読込み、GMP有理数で元制約・境界・整数性・目的値を確認する。

出典: https://miplib.zib.de/downloads/miplib2017-testscript-v1.0.4.zip
アーカイブSHA256: 37b21a7f1ee968aed60675fb070c9286eccc6a953c8bc50120c9246b052b47b9
元archiveのchecker/srcとchecker/Makefileを変更せず保存。MIT、Copyright 2019 Zuse Institute Berlin、LICENSE参照。

構築: `make -C third_party/miplib-checker CXX=g++ LINKCXX=g++ USROFLAGS=-O2`
実行: `bin/solchecker original.mps.gz solution.sol 1e-6 1e-6`

第3引数はlinear_tol、第4引数はint_tol。linear/bound検査はpospart/negpart/side/1でスケールする相対検査 (src/model.cpp参照)。exit code 0だけでは合格と判定せず、Read MPS/Read SOLとCheck SOLの3項目を確認する。現評価のprimaryは別の絶対誤差Python検査で、この公式検査は補助として両方の結果を保存する。

本checkerには入力形式上の制約がある。短すぎるfree形式名の一部や目的定数RHS等を読めない場合は、解の誤りとせず検査不能として記録する。公式Benchmark Set v2全240問には非ゼロ目的定数RHS/OBJSENSEがないことを別途監査済み。
