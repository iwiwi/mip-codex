# 基底右辺を列ベクトルviewから1列行列へ変更する仮説

2026-11-16。mainはPR67の1143bda01（基底コピー削減、F0c8fbf36c1facc49）。MIR再版は486c7da16e0584a8であり、MIR無効のbinkar根LP12組でも.08463→.10635秒と退行した。MIR再版の36出力571切断と26保存MIP点は全てFraction/GMP/公式検査を通過したが、性能退行を採用しない。ソースとバイナリをmir-inplace-snapshot、旧初期版をmir-initial-snapshotへ固定保存済み。

faer0.24.4（既に採用している汎用線形代数ライブラリ）のcol/colref.rs `ColRef::as_mat`は列間stride0を返す。perm/mod.rs `permute_rows`はrow_strideとcol_strideの大小でループを選択する。1列のCol viewだと各行の長さ1のcopy_fromが呼ばれ、インライン化しない場合は多数の短い関数呼出しが残る。MIR再版の短いperf samplingはfaer Col copy23%、命令数約1.420B（main約1.246B）だった。新機能が汎用関数のコード生成を変える可能性を示すが、LLVMの内部原因全てを確定したとは扱わない。

別targetでCODEGEN_UNITS=1のMIR試作は根LP中央値.07899秒まで短縮した。これはsource fingerprintには環境profile overrideが含まれない明示的な診断ビルドで、設定として未採用。依存コードを改造/移植せず、Basisの一時右辺をfaer Mat(m,1)にすれば非ゼロ列間strideを保って同じ線形系を解ける。この入力形式の変更をmainから独立に評価する。PR67のin-place solveは維持する。

LP選択・係数・LU因子・eta更新・許容差は変えない。全202 Rustテスト、純根LP12組の時間以外の全出力比較、固定18問の同一CPU対比較、合成/LP oracle/元MPSへの独立解検証を実施する。改善と正当性が確認できれば採用し、その後MIRを再度載せる。失敗も別記録として保持する。
