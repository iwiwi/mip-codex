# Binary feasibility pump試作

2026-11-15。整数変数が全てbinaryまたは元境界で固定の問題を対象に、根LP点を丸め、丸め点へのL1距離を自作Rust LPで最小化する。binary z=0なら+x、z=1なら-xで距離目的が線形になるため、行列/境界を変えずwarm cost changeを再利用できる。連続変数は距離に含めない。一般整数の補助変数版は初期試作の対象外。

考え方の出典：Fischetti/Glover/Lodi, The feasibility pump, November 2003 / May 2004 draft, https://miplib2010.zib.de/miplib2003/paper/fischettigloverlodi2004.pdf （2026-11-15閲覧）。大学の書誌 https://www.research.unipd.it/handle/11577/2431994 、出版DOI 10.1007/s10107-004-0570-3。数式上の方法を参照し、既存のコードはコピー/委譲しない。

候補だけを元モデルで検査してincumbentへ渡す。変更した目的値/dual/不能結果を本探索の枝刈りや下界に使わない。親warmは不変で、局所warmのみ更新する。根だけ、最大64投影、各4096ピボット、時間の10%かつ最大1秒。直近8丸め点との重複時は固定seedで分数性を優先した小さい反転集合を作る。これらは汎用上限であり問題名/既知解を利用しない。

既定無効で0/1比較する。小規模全列挙・混合binary/連続・元問題検査・期限/不能を本探索へ流さないことを確認し、固定開発18問で有効性を判断する。未知検証用問題では調整しない。
