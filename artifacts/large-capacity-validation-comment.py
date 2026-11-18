from pathlib import Path
import os,json,subprocess
os.sched_setaffinity(0,{43})
assert json.load(open('artifacts/large-capacity-default-development-parallel-audit/summary.json'))['fresh']==376
text='''既定ONの最終ソース1572f4382c/FPabc0956a38a27666を凍結し、350 Rust成功/4ignored、200小規模1200出力の全列挙・元MPS GMP・厳密証明、7410改変棄却、40大規模解析例のGMP・証明・最適値一致と160改変棄却を確認しました。深さ制御4点も元GMP/strict proofが成功し、1020個5-cycleでは512節点でdepth_limit、安全上界3060/主解2040、単一5-cycle＋孤立点では2節点で正しくoptimalでした。深さを想定した最初のテスト例は実際には早期optimalになり、テスト期待の誤りを訂正。初回source/logを保存し、optimizerは変更せず両ケースを追加検査しました。

最終188問10秒比較はprimal185同じ/0改善/3悪化、dual173同じ/0改善/2悪化/13比較不可、finite175ずつ、statuses両182time/2optimal/4numeric。全376出力を新規strict証明検査、128点は元GMP＋公式検査valid（再利用0）。新しい大容量graph認識は0件で、map10・neos-911970・supportcase12のprimal悪化、seymour1・sp98arのdual悪化を保存しています。初期FPf293ではprimal187同じ/1改善/0悪化、dual173同じ/1改善/1悪化/13比較不可でした。いずれも改善を本機能へ帰属しません。最終の悪化5件は同じ凍結ABC・CPU46で3交互順ペアずつ再測定を開始し、再現性の検査を待つためDraftを維持します。

最終の主制御7点は元MPS GMPと独立root/prefix/Fraction証明が全valid。z26/60秒・CPU44・16GiB1threadはOFF−7/LB−1651→ON−1154/LB−1578、ON73,083節点、depth_limit_hit false。初期FPf293のLB−1575と混同せず、時間制限内のroot被覆構築の変動も残します。cod105は両optimal−12・LP1/13pivot・約.0208秒、S3は両optimal−16/6042153節点。native60秒は既報のHS−1147/−2114、SCIP−1013/−1650.9375（2点GMP valid）、GLPK解/境界なしを同入力SHAの新規比較として保持。この単例を全体優位/新記録とは扱いません。

初期評価7222ファイル/429,310,393Bの5bundle全payload SHA照合と除外ELF保持、前PR126 postmerge検査・未起動snapshot302b、private原型5点GMPと公式MPS取得記録をcommitしました。全Rust最終240は未起動。baseline719/720は凍結管理プロセスの復旧後も稼働GLPKを維持し、再最適化せず続行中です。
'''
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True)
Path('artifacts/large-capacity-validation-ai-usage.json').write_text(usage)
p=Path('artifacts/large-capacity-validation-comment.md')
p.write_text(text+'\nその時点のAI資源量:\n```json\n'+usage+'```\n')
r=subprocess.run(['gh','pr','comment','127','--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True)
Path('artifacts/large-capacity-validation-comment.log').write_text(r.stdout+r.stderr)
r.check_returncode()
print(r.stdout)
