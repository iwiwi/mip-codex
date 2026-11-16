# 原問題3行集約: 根カットの質と時間依存の主解

元MPSの異なる2行だけでは自由連続変数が残る場合に、異なる3行の正整数結合で消去し、既存の厳密CG丸めを適用する。例えば3x+2t≤3、3u−t≤0、3z−6u≤1を重み1,2,1で足すと3x+3z≤4となり、整数x,zにはx+z≤1が有効。問題名/既知値/holdoutを条件にしない。正整数重み、元の行側・変数境界、checked i128、最終2^52以内を検査する。Python Fractionは元MPSから3行を再構成する。

既存cut/MIRの後に最大8本の試行LPを自作Rustで解き、期限内の相対目的改善1e-3以上または有効整数点の場合だけ保持する。失敗/未完了/試行infeasibleでは元のモデルprefix/cacheへ戻す。trial infeasibleを大域実行不能へ昇格しない。追加の分離探索は1M work・20msの目安で、sort等を含む実時間が20msを越えた例もrawに残す。根cut全体は既存のmin(全時間5%,0.25秒)を維持する。

3行cut保持時だけ、cut直前のモデルと不変warm基底を既存の1秒・4096node主解probeへ渡す。branch/order/strong/objective-gridを外側と揃え、localのbound/statusは昇格せず、候補だけ元問題で再検査する。precut probeなし版ではtimtab60秒1125560に留まり、現行版1072270の方が良かった。一方10秒の主解悪化はこの変更では消えなかった。

## 同一binaryによる比較

各1thread・16GiB AS、同CPUでon/offの順序交替。すべて開発188問題。

| 比較 | 主解 | dual | 元MPS独立検査 |
|---|---|---|---|
| 旧LP基準FPddf7・10秒 | 0改善/1悪化/187同値 | 3改善/0悪化/75同値/110比較不能 | 376証明・108整数点有効 |
| 旧LP基準FPddf7・60秒 | 1改善/0悪化/187同値 | 3改善/0悪化/94同値/91比較不能 | 376証明・118整数点有効 |
| PR73統合FP337114ffc2f9a6b7・10秒 | 0改善/1悪化/187同値 | 3改善/0悪化/76同値/109比較不能 | 376証明・110整数点有効 |

10秒のtimtab1は主解1089832→1166536と悪化する。60秒では1089832→1072270へ改善。dualは両方148854.431842→150732.537106。enlight_hardは主解なしのままdual16→16.458333、sp150x300dは主解71同値・dual27.3014103→30。短時間の主解悪化と長時間の改善を両方残す。全体の速度や解けた件数が改善したとは述べない。旧60秒の最適宣言2件は前後同数で、点の独立検査済みという集計であり探索全木の再演算ではない。未解決60秒ペナルティSGMは59.5206013→59.5226242。

統合10秒はbefore time_limit172/resource_limit11/numerical_error5、after171/11/6。差はswath1で同じ主解448.882839・dual334.496858を保持し、3行cut採用なしのままdeadline付近に数値終了した。旧版の再比較は両方numerical_errorとなったが、正式なraw差は消さない。統合10秒の最適宣言0件、未解決10秒ペナルティSGMは両方10秒。

統合版の選択6問題60秒も比較し、timtab/enlight/sp150の改善を再現、neos860300は3966/2031、germanrrは主解なし/45980135.4164、nu25は54375/52975を維持した。12出力の証明と8整数点がGMP絶対/公式とも有効。214 Rust、2700合成、根LP300設定500出力、数値32件（18有効点+14精度未解決）が通過。LP oracleは3行cutを明示的に無効にして純LP比較を保つ。既定変更時の比較unitはbeforeのfalse指定漏れで一度失敗し、実装を緩めず基準設定を明示して修正した。

## 主workspace再現とnative

FP337114ffc2f9a6b7を主workspaceで再build。実run smoke-20261116-090230-s812zo5i、同CPU41・60秒・1thread・16GiB・同一timtab1。

| solver | 主解 | dual | raw status |
|---|---:|---:|---|
| Rust | 1072269.99999977 | 150732.537105506 | time_limit |
| HiGHS | 764771.99999988 | 764771.99999988 | Optimal |
| SCIP | 764771.99999988 | 764771.99999988 | optimal |
| GLPK | 1147266 | 417614.233906926 | feasible_limit |

自作はGLPKより主解だけ良く、dualは弱い。HiGHS/SCIPは最適まで解いており両指標で及ばない。別10秒run smoke-20261116-090601-1miwkuhoのbinkar6888.550001/6646.313678834、nidda14054.945596895/−1216923.270104406を維持した。niddaは依然numerical_errorであり完走ではない。全体優位・世界記録・全240評価完了は未達成。

現在の採用候補branchだけ既定true。mainへの採用時にはMIP_THREE_ROW_CUTS=0で停止できる。十進行MIRは別branchで188対主解0改善/1悪化となり未採用。本変更へ混ぜない。全240/3600秒snapshotはcreate-onlyで、最終評価開始を11月19日UTCより遅らせない。

主workspaceの2実run計6点はGMP絶対許容差・公式とも有効で、3 Rust出力のcut/grid証明も有効。主workspaceのsource/binary originを照合し、aggregate-three-final-20261116の240 job snapshotをcreate-onlyで作成、全hashを検証した。experiments/aggregate-three-snapshot-20261116.jsonに記録する。正式全件ジョブはまだ開始していない。
