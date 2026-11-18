from pathlib import Path
import subprocess,json,datetime
root=Path.cwd()
def run(args):return subprocess.run(args,check=True,capture_output=True,text=True).stdout
assert not run(['git','diff','f62a4a91fc','--','src','Cargo.toml','Cargo.lock','build.rs'])
a=json.load(open('artifacts/pump-limited-default-development-parallel-audit/summary.json'));assert a['completed']==376 and a['strict_valid']==130 and a['strict_invalid']==0 and a['cut_invalid']==0
r=json.load(open('artifacts/pump-limited-default-development-analysis.json'));assert r['comparisons']==dict(primal_equal=186,primal_improved=2,dual_equal=173,dual_improved=2,dual_incomparable=13)
for p in Path('artifacts/pump-limited-default-development-screen/cases').glob('*/*/job.json'):
 j=json.load(open(p));assert j['split']=='development';env=j['environment'];assert env.get('MIP_PUMP_LIMITED_ROUNDING')==('0'if p.parent.name=='before'else None)
run(['python3','artifacts/pump-limited-archive-preflight.py'])
head=run(['git','rev-parse','HEAD']).strip();assert run(['git','ls-remote','origin','refs/heads/pump-limited-rounding']).split()[0]==head
body="""pump補助LPが時間・反復上限で終了した際、その点を元変数へ写し、整数座標だけ丸め、既存の元行可行性検査を通る場合だけ主解として回収します。補助LPのdual/basis/最適性は使いません。全体期限確認を保持し、LP予算・乱数・制約・許容差を変えません。既定ON、MIP_PUMP_LIMITED_ROUNDING=0で無効化できます。

初期core2d5725/FP4029と既定ON coref62a4a/FP7f77を別凍結し、Rust/Cargo/build差は既定boolだけと確認。各Rust357 passed/4ignored。初期binary200全成功、既定ON一般200/binary200全成功。両mixed200は195成功/5 numerical_errorです。5件はmain129・候補OFF/ONで同じ保守終了、pump.calls=0。初回5点+再現15点+既定ON5点の元GMP検査と再現15+既定ON5のproof検査が成功、5問のFraction参照を保存。既存の未完了を成功扱いせず、guard/許容差は維持します。

開発188問、各10秒/16GiB/1thread、同一凍結CLIのOFF対ON。初期は376fresh proof/129点GMP公式valid、主解187同等/1改善、dual174同等/1改善/13比較不能。既定ON・環境変数未指定の最終は376fresh proof/130点GMP公式valid、主解186同等/2改善、dual173同等/2改善/13比較不能。両182time_limit/2optimal/4numerical_error、有限下界175ずつ。mcschedのNone→309595.0000000017は新回収1候補/1改善で再現。graph20のNone→0は回収拒否後の変動、seymour1/sp97arの下界変動はpump未実行で、成果に帰属しません。

mcsched60秒のOFF/既定ONは両266672/LB193774.75370663218、両点GMP/proof valid。利点は10秒での早期可行点発見で、60秒の最終品質差は確認できません。同条件10秒のnativeはHiGHS1.16.0 212889、SCIP10.0.2 212887、GLPK5 216929（全点GMP valid）で、本候補は全てより劣ります。全体優位・新記録は未達。

4archiveの全8227 payloadと除外2ELFを元の物理保存先でSHA再検証。小規模profile名の誤記、60秒driverのoptimizer開始前path失敗も保持・訂正。holdout52を判断に使わず、正式240問は未起動です。
"""
p=Path('artifacts/pump-limited-pr-final-body.md');p.write_text(body);q=Path('artifacts/pump-limited-pr-final-payload.json');q.write_text(json.dumps(dict(title='pump補助LPの打切り点を元問題の可行主解として回収',body=body)));run(['gh','api','--method','PATCH','repos/iwiwi/mip-codex/pulls/130','--input',str(q)])
comment='最終検証を完了し、既定ONで採用します。\n\n'+body
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);Path('artifacts/pump-limited-adoption-ai-usage.json').write_text(usage);p=Path('artifacts/pump-limited-adoption-comment.md');p.write_text(comment+'\nその時点のAI資源量:\n```json\n'+usage+'```\n');out=run(['gh','pr','comment','130','--repo','iwiwi/mip-codex','--body-file',str(p)]);Path('artifacts/pump-limited-adoption-comment.log').write_text(out);print(out,flush=True)
v=json.loads(run(['gh','pr','view','130','--repo','iwiwi/mip-codex','--json','headRefOid,isDraft,state']));assert v['state']=='OPEN'and v['headRefOid']==head
if v['isDraft']:run(['gh','pr','ready','130','--repo','iwiwi/mip-codex'])
run(['gh','pr','merge','130','--repo','iwiwi/mip-codex','--squash','--match-head-commit',head]);v=json.loads(run(['gh','pr','view','130','--repo','iwiwi/mip-codex','--json','headRefOid,state,mergeCommit,mergedAt,url']));assert v['state']=='MERGED'and v['headRefOid']==head;Path('artifacts/pump-limited-pr-merge.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v),flush=True)
