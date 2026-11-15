# MIPLIB 2017公式solution checker

検証専用の独立プログラム。Rustソルバーから呼ばず、最適化をしない。元MPSを独自readerで読込み、GMP有理数で元制約・境界・整数性・目的値を確認する。

出典: https://miplib.zib.de/downloads/miplib2017-testscript-v1.0.4.zip
アーカイブSHA256: 37b21a7f1ee968aed60675fb070c9286eccc6a953c8bc50120c9246b052b47b9
元archiveに対しMIPLIB_ABSOLUTEマクロで分離した検査拡張を追加。通常ビルドの検査式は原版を保持。MIT、Copyright 2019 Zuse Institute Berlin、LICENSE参照。

構築: `make -C third_party/miplib-checker CXX=g++ LINKCXX=g++ USROFLAGS=-O2`
実行: `bin/solchecker original.mps.gz solution.sol 1e-6 1e-6`

第3引数はlinear_tol、第4引数はint_tol。linear/bound検査はpospart/negpart/side/1でスケールする相対検査 (src/model.cpp参照)。exit code 0だけでは合格と判定せず、Read MPS/Read SOLとCheck SOLの3項目を確認する。新規評価のprimaryはGMP有理数による絶対誤差検査 (solchecker-absolute)。Python浮動小数点検査は診断のみ。通常版の公式相対検査も補助として保存する。既存baseline snapshotは変更せず、保存解を新検査で再監査する。

本checkerには入力形式上の制約がある。短すぎるfree形式名の一部や目的定数RHS等を読めない場合は、解の誤りとせず検査不能として記録する。公式Benchmark Set v2全240問には非ゼロ目的定数RHS/OBJSENSEがないことを別途監査済み。

絶対検査版の構築:
`make -C third_party/miplib-checker MAINNAME=solchecker-absolute OBJDIR=obj-absolute CXX=g++ LINKCXX=g++ USROFLAGS=-O2 USRCXXFLAGS=-DMIPLIB_ABSOLUTE`

絶対版は境界・行を非スケールの絶対許容誤差で判定し、目的値許容誤差は `1e-8*(1+abs(computed objective))`。元の十進係数と解の十進値をGMP有理数で演算する。短名free形式、目的定数RHSに対応し、厳密な目的値と違反量を出力する。Python側で変数名の完全一致を別途確認する。最適性証明ではない。主検査が利用不能なら検証不能とし、浮動小数点結果だけで合格にしない。

MIPLIB_ABSOLUTE版の数値入出力は可変長。元の32文字assertを外し、長い正確な十進係数・E/D指数をGMPへ渡す。数値token/指数には資源保護上限を設け、未対応tokenは検査不能とする。通常版は原版の制限を保持し、稼働snapshotのbinも変更しない。
