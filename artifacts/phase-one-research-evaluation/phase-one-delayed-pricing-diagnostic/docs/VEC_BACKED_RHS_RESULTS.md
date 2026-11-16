# Vecを直接使うLU RHSのコピー削減

Basis.forward/backwardで、疎LUの前後に行列とVecを確保・コピーしていた。Vecをn行1列、column stride=nのMatMutとして直接解くことで、forwardの出力収集とbackwardの往復コピーを除いた。Colのstride0回帰は再導入しない。数式・探索規則・許容誤差・時間/仕事量上限は変えない。候補FP5a6ccff29edba788、比較元main FP527633393c003557。

## LP時間と正しさ

同CPU・1thread/16GiB・旧新を交互に実行し、純LPに分離した。時間以外の返却JSONは全対で完全一致。

|開発LP|対数|before中央値秒|after中央値秒|短縮|
|---|---:|---:|---:|---:|
|binkar10_1|20|0.0874986965|0.083969019|4.03%|
|germanrr|3|19.59370202|18.552188209|5.32%|
|neos-860300|3|15.150955437|15.000176117|1.00%|
|neos-911970|10|0.006924733|0.0068384545|1.25%|

全36対で候補が短く、ピボット・目的・dual・状態も同じ。72保存連続点は元MPSのFraction検査に合格。整数実行可能点とは数えない。profilesの995 hardware samplesでMatコピー等が現れたことが着手理由だが、sampling付き時間を性能得点には使わない。プロファイルの元ELFはparity-equality-final snapshotにSHA一致で残す。

219 Rust、通常2700合成、LP300設定500出力、数値32が成功。数値32は18有効点/14精度未解決。unitsの初回100件は疎worktreeにchecker実行ファイルがなく、パス指定も落として未検証になった。同じELFへ正しいMIP_ABSOLUTE_CHECKERを設定して100件すべて成功。初回の失敗ログを保全し、正常500出力とは分ける。

## MIPの全188開発対照

各10秒、同CPUで旧新を交互測定。主解1改善/1悪化/186同値、dual1改善/0悪化/78同値/109比較不能。両方170 time_limit/11 resource_limit/1 optimal/6 numerical_error。376出力の切断・格子証明、112整数点のGMP絶対/公式検査は全件有効。唯一の既存最適解enlight_hardは双方とも独立一意性証明付き37。SGM(shift1/未解決10秒)はほぼ同じ9.860595であり、MIP全体の速度優位は示していない。

kasaiの主解1072846.63→1072833.43、loueのdual22956→22966。neos-911970の主解は111.84→144.23に悪化し、60秒でも同じ差が残る。根LPの全返却内容はこの問題でも完全一致し、probe無効の別10秒対照は両版148.86。1秒のroot probeは旧1263/1275node付近、新1348nodeへ進み、RINS固定数も735→733へ変化した。時間予算のある局所探索の経路変化が関係する観測であるが、悪化を補完・消去しない。

60秒10問では、loueのdual22953→22966が再現し、kasaiは双方1072826.39で同じ。上記neos悪化以外のbinkar、nu25、timtab、neos-860300、germanrr、nidda、sp150は主解/dual同じ。probe無効対照を含む22出力/18整数点も全件有効。

採用理由は、余分なコピーを安全に除き、反復LPを同じ入出力のまま1–5%短縮したこと。MIPの品質が普遍に改善するとは主張しない。残る主解悪化を既知の制約として示し、問題別の時間補正や設定は埋め込まない。

## 主workspaceの再測定とnative比較

同FPを主workspaceでビルドし、同CPU41/10秒/16GiB/1threadで測定。全てmin問題。

|問題|solver|主解|dual|状態|
|---|---|---:|---:|---|
|kasai|Rust|1072846.63|945808.0956|time limit|
|kasai|HiGHS1.16.0|988585.62|975527.2155|time limit|
|kasai|SCIP10.0.2|1011086|969138.2068|time limit|
|kasai|GLPK5.0|1029059.04|976124.6943|feasible limit|
|neos-911970|Rust|144.23|23.26|time limit|
|neos-911970|HiGHS1.16.0|54.76|54.76|optimal|
|neos-911970|SCIP10.0.2|54.76|54.76|optimal|
|neos-911970|GLPK5.0|60.68|23.4124|feasible limit|

Rustは両問題とも全nativeより主解/dualが弱い。kasaiの小さい10秒主解改善は主workspaceでは再現しなかった。binkar6888.550001、nidda14054.945596895、enlight37最適の追加対照は従来品質を維持。計11保存点と5Rust出力23切断・格子証明は全て有効。

## 保存と次の作業

原出力、誤ったchecker設定の失敗、source/ELF receipt、全snapshot作成記録はartifacts/vec-backed-rhs-evaluation。ELFとsampling生stackはローカル保持しSHA参照、symbol集約reportを保存。分母不一致後の再分解案はnidda主解悪化で未採用、8出力/4点の検査とsourceをsupporting-recordsへ残す。

全240/3600秒のvec-backed-rhs-final-20261116はcreate-only、全hashを照合済み。最終自作全件は未開始、全体native優位性や新記録は未達成。全188開発だけの構造分類を別診断で完了し、次は元整数行の同一係数・範囲支配による冗長行の削減を検討する。holdout入力は使わない。11月19日UTC目安で最終自作全件を開始し、11月22日14:59UTC期限を守る。
