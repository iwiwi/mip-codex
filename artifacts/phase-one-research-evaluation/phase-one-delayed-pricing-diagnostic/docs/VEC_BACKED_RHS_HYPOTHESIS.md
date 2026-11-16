# 非零column strideを保つVec直結LU RHS

2026-11-16、main4556e45d6/FP527633393c003557から。hardware perf cycles:u/99Hz/dwarfでbinkar10秒の995 samplesを採取（計測オーバーヘッドがあるため性能得点には使わない）。run_priced13.65%、warm_impl7.01%、faer row permutation5.75%、Mat copy4.27%、forward/backwardのMat→Vec収集が計約2.95%。cpu-clock samplingはPMU未対応で失敗、失敗ログも保存。

Basis.forwardは従来Matを確保し入力をコピー、LUsolve後に別Vecへ収集していた。backwardは既に持つVecからMatへさらにコピーし、新Vecへ戻していた。MatMut::from_column_major_slice_mutを使い、出力Vecをn行1列の非零column stride=nの行列viewとしてそのまま解く。PR68で避けたCol viewのstride0を再導入しない。数式/探索規則/許容誤差/予算は変えない。

次: 凍結main対候補の根LPで共通JSON/ピボット/目的/点を照合し、同CPU交互反復で内部時間を比較。改善信号があれば219 Rust/2700合成/500根LP/数値32と元MPS点検査、全188/60秒へ進む。main全240snapshotはparity-equality-final-20261116を作成/照合のみ。holdout入力は使わない。

候補FP5a6ccff29edba788をvec-backed-rhs-v1-diagnosticへ全ソース/ELF/SHA/build-infoとともに凍結。binkarの同CPU40・交互20対は内部中央値0.0874986965→0.083969019秒、比0.959660（約4.03%短縮）。germanrr同CPU40・3対は19.59370202→18.552188209秒（約5.32%短縮）、neos-860300同CPU41・3対は15.150955437→15.000176117秒（約1.00%短縮）。全対で時間欄を除く共通JSONが完全一致し、ピボット/目的/dual/状態も同じ。

219 Rustを成功。数値32（18有効点/14精度未解決）も成功。通常2700合成・LP500出力・全188開発10秒と独立監査は実行中。旧版は同じworktree由来の凍結main FP527、両方parity ON（純LPだけOFF）、同CPU・1thread・16GiB・元入力SHA照合。CPU3..7 screen、CPU39 audit、CPU2 synthetic、CPU1 LP oracle。結果はprimary artifacts/vec-backed-rhs-*、worker-*-exit.jsonとprofile/summaryから再開できる。main全件評価は未開始のまま。

追加結果: 根LP52保存点は元MPSのFraction検査で全件有効（連続点であり整数実行可能点とは数えない）。各LP反復はbinkar20/20、german3/3、neos3/3で短かった。通常合成2700とordinary/scaled LP400出力を通過。units初回はcheckerパス指定を落とし、疎worktreeにはcheckerのソースだけで実行ファイルが無く100件が未検証だった。失敗ログを残し、同ELFにMIP_ABSOLUTE_CHECKERを正しく指定して100件すべて成功。正常500出力と初回の無効な検査実行を分けて記録する。
