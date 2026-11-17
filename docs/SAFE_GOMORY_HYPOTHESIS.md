# 厳密整数演算で証明を残すGomoryカットの研究

2026-11-17。親PR102。現時点は設計のみで、mainへカットを追加していない。

一次資料: Eifler/Gleixner, Safe and Verified Gomory Mixed Integer Cuts in a Rational MIP Framework, arXiv:2303.12365v3 (2023), https://arxiv.org/abs/2303.12365v3 、https://arxiv.org/html/2303.12365v3 。浮動小数LPから近似乗数を得ても、その後の行集約と丸めを安全に行うという考え方を参照する。2022年SCIP講演 https://scipopt.org/20years/slides/eifler.pdf 。SCIPのGMI例の数式説明 https://scipopt.org/doc/html/sepa__gmi_8c.php も参照。著作権のある本文やコードは転載せず、最適化処理を外部へ委譲しない。

最初は全変数・全対象行係数/境界が厳密整数の場合に限定する案。各元行の有限上側を a_i x+s_i=b_i（下側だけなら符号反転）と書く。s_i>=0は整数。等式行のslackは0。基底のBTRANから得る近似乗数を一定二進格子へ丸め、整数N_iとする。以降の演算はchecked i128またはBigIntで厳密に行い、overflowや表現できない係数は棄却する。

A_j=ΣN_i a_ij, B=ΣN_i b_i。選ぶ整数変数kについてM=A_k>0となるよう全Nの符号を揃える。M=0なら棄却。各元変数は整数の有限下限l_jによりz_j=x_j-l_j>=0、または有限上限u_jによりz_j=u_j-x_j>=0へ変換する。anchor_jをl/u、direction_jを+1/-1とし、B'=B-ΣA_j anchor_j。

R_j=(direction_j A_j) mod M、S_i=N_i mod M、R_0=B' mod M（全てEuclid剰余）。純整数Gomoryの妥当不等式は ΣR_j z_j+ΣS_i s_i>=R_0。元空間へ戻すと係数 C_j=direction_j R_j-ΣS_i a_ij、右辺 D=R_0+Σdirection_j R_j anchor_j-ΣS_i b_i で Cx>=D。基本変数の係数は剰余0になるが、近似乗数が正確な逆行列である必要はない。全ての元行等式と整数性から導けるため、近似は分離性能だけに影響する。この代数は独立したFraction実装と全列挙で検証する。

最初は全C,Dが厳密f64整数範囲へ収まる場合のみ出力する。GCDで縮められる場合も厳密に行う。数値係数を丸めて強めたり、わずかな違反を正しい証明とみなしたりしない。証明には元行side、N、k、anchor方向を保存し、Python検査器は元MPSから全演算を再構築する。既存のcut proof機構を拡張し、改ざんを棄却する検査を入れる。

接続で未解決: WarmStartの基底はscaling::solveで列/行スケールをかけた問題に対応する。元row_sourcesへBTRAN乗数を戻す処理が必要。Standard.shift/termsには元変数の置換がある。乗数の精度を証明には使わないが、分離のためには適切な単位へ戻す。元モデル全変数の整数性・係数/境界のexact flagsを判定する。まず独立導出と小規模検査を完成し、それからLP基底との接続を行う。

既存mainのMIRは元の単一行、aggregateは2/3行の正整数集約であり、基底行からの一般集約は未実装。この不足を補う仮説。採用判断は独立検証と開発例の有効/無効比較で行い、検証用52例には触れない。

独立Python/Fraction代数検査: seed2026111703、500生成モデル、実行可能整数点15616個、非零fractional RHS471例、違反0。負の元変数境界、負の行係数、正負の乗数、上下限へのcomplementを含む。artifacts/safe-gomory-algebra-oracleに入力・導出・結果を保存。これは分離性能ではなく妥当性だけの検査。
