# 整数値shiftに対する元係数誤差の計算を簡約する仮説

2026-11-15のgen-ip002開発profile（cpu-clock99Hz、313sample、lost0、1thread/CPU7/16GiB）ではRow::activity_errorが17.25%、Basis::backwardとwarm_implが各11.82%、standard_sparseが7.03%だった。5秒上限でも実際には既存の未証明LP不能を検出して37993節点でnumerical_errorとなる。主解-4729.7692539/dual-4840.54196130089は元MPS検査に合格したが、最適性や不能証明の成功ではない。profileの割合だけから速度改善を推定しない。

standard変換はshift全成分についてpoint_error=0かを既に調べている。これは現行serializeで厳密な整数として保存する有限binary64整数値の点である。全成分のdx=0と分かっている場合、行の活動誤差の三項 da*abs(x), abs(a)*dx, da*dx のうち後二項は常に0。x=0の項は残る第一項も0なので係数の誤差計算自体も不要になる。

Rowにこの条件専用の実装を追加し、shifted_boundから既存exact_shiftがtrueの場合だけ使う案を次に検証する。非整数点は旧実装を使う。誤差の閾値や丸め順は変えず、各非零項について同じadd_errorを同順で実行する。小数係数の元区間を無視する案ではない。係数誤差daは非零shiftで引き続き計上する。

採用前に旧式とのbit一致検査を、符号・0/-0・大整数・subnormal係数・係数のexact flags・overflowを含めて実施する。同じ37993節点のgen-ip002固定作業で3反復、固定開発問題と元MPS検査、数値回帰を比較し、失敗状態もそのまま残す。生profileはartifacts/revised-profile-nextに保存してあり、次のPRの実験archiveへ移す。
