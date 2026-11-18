from pathlib import Path
import json,subprocess
root=Path.cwd()
def run(args):
 r=subprocess.run(args,capture_output=True,text=True)
 if r.returncode:print(r.stdout,r.stderr,flush=True)
 r.check_returncode();return r.stdout
assert not run(['git','diff','4d4b1b5e3a','--','src','Cargo.toml','Cargo.lock','build.rs'])
for f in ['src/binary_graph.rs','src/binary_graph/weighted_cover.rs']:assert run(['git','show','50fe54699c:'+f])==Path(f).read_text()
a=json.load(open('artifacts/cover-long-round-default-development-parallel-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['strict_valid']==a['official_valid']==136
assert json.load(open('artifacts/cover-long-round-default-main-controls/exit.json'))['gmp_valid']==9
assert json.load(open('artifacts/cover-long-round-default-long-controls/exit.json'))['gmp_valid']==4
assert json.load(open('artifacts/cover-long-round-hour-controls/exit.json'))['proof_valid']==2
for p in Path('artifacts/cover-long-round-default-development-screen/cases').glob('*/*/job.json'):
 j=json.load(open(p));assert j['split']=='development'and j['threads']==1 and j['memory_bytes']==16*1024**3;assert j['environment'].get('MIP_GRAPH_COVER_LONG_ROUNDS')==('0'if p.parent.name=='before'else None)
run(['python3','artifacts/cover-long-round-archive-preflight.py']);head=run(['git','rev-parse','HEAD']).strip();assert run(['git','ls-remote','origin','refs/heads/graph-cover-long-rounds']).split()[0]==head
body='''大規模疎packingで既存heapが実際に有効、かつ64bitの場合に限り、改善後のweighted局所探索を2Mから100M反復へ延長します。初期無改善時の上限は変えず、改善後の既存restart系列も最大8回のままです。既定ON、MIP_GRAPH_COVER_LONG_ROUNDS=0で従来動作。全体/局所期限・memory gate・RNG・認識・上下界・枝刈り・許容差は変更しません。旧公開APIはfalse wrapperを保持します。9roundの計数/score/weightの整数上界を別証明し、100Mでもoverflowしないことを確認しました。

初期50fe/FP2065はmain129上で353Rust/4ignored。最終4d4b1b/FPcbfaはmain131を統合して360Rust/4ignoredです。初期と最終のgraph2ファイルは同一、main131統合後a721→4dのRust/Cargo/build差は既定boolだけ。最終凍結CLI SHA9b3a119dcc666c2ad0eb4f0efd4e4740a5574d554a5485535825f6447fca7741。

両段階とも1200小出力の全列挙/GMP/strict proofと7410改ざん拒否、12large/core独立列挙/GMP/proofに成功（長mode実行6出力）。最終largeは主解6組同じ、upper4同じ/1改善/1悪化も保持。最終9controls+4longは13点GMP/proof valid、cod105根最適−12、5000pairs根最適−2500、S3両−16/6042153nodes最適を維持。

最終同一CLI、1thread/16GiB、OFF/環境変数未指定ONの結果:

| 問題/制限 | OFF primal/dual | ON primal/dual |
|---|---|---|
| z26/60秒 | -1178/-1575 | -1183/-1575 |
| CDC/60秒 | -279/-434 | -279/-429 |
| z26/300秒 | -1187/-1578 | -1193/-1578 |
| CDC/300秒 | -285/-434 | -286/-424 |

初期source50feの1時間対照は同一凍結CLI3222/CPU46で、OFF−1188/LB−1578（3600.042092849秒）→ON−1196/LB−1575（3600.039068732秒）、両点GMP/strict proof valid。ONは189054109反復/899.399733885秒の局所探索を実施。最終sourceの1時間結果と同値再現したとは主張せず、初期と合成後の評価を区別します。

同マシン3600秒/16GiB/1threadのz26はHiGHS1.16.0−1147/LB−2114（3600.046330秒）、SCIP10.0.2−1070/LB−1533.488527（3600.006063秒）、両点GMP valid。候補の主解は良く、SCIPの下界が強い。GLPK5は3660.012435秒のwhole-process-group watchdog/rc−9、時間内受理点なし、該当live processなしを確認。全失敗を保持します。CDC300の既存同条件nativeはHS−280/LB−1111（306.729989秒）、SCIP−237/LB−11744（300.005887秒）、両GMP valid、GLPK420.007秒watchdog失敗で、候補はこの入力の主解/下界が良い結果です。

最終開発188問は376fresh proof/136点fresh GMP公式valid。primal186同等/1改善/1悪化、dual173同等/1改善/1悪化/13比較不能、両182time_limit/2optimal/4numerical_error、有限下界175。全認識graphを独立確認してcod105/S3の1024変数だけ、長mode発動0でした。mcsched314588→313589改善、kasai1072833.43→1072846.63悪化、seymour1下界改善、sp98ar下界悪化を全保持し成果に帰属しません。追加Kasai3pairsは各1072833.43同じ、sp98ar3pairsも各組同じ下界（12点GMP/proof）ですが元損失を置換しません。初期188の3主解改善とn3div36下界悪化も非発動時変動として保持。

15archiveの14251payload SHAと除外12ELFを元の物理保存先で再確認。sparseで省略されたtracked recipe/archivesはGit blobから読み、最初のrecipe読込失敗はoptimizer再起動なしで記録しました。私的1198 tieと頂点penalty探索の不改善も別sourceの結果として保持し、本候補/mainの記録達成に帰属しません。比較器の点はRustへ渡していません。

初期1時間と合成後300秒の主解改善、全独立検査を根拠に採用します。正式240問は未起動、holdout52を調整に使わず、全体優位/新記録は未達です。
'''
p=Path('artifacts/cover-long-round-pr-body.md');p.write_text(body);out=run(['gh','pr','create','--repo','iwiwi/mip-codex','--base','main','--head','graph-cover-long-rounds','--draft','--title','大規模疎packingの改善後weighted探索を長roundに延長','--body-file',str(p)]);Path('artifacts/cover-long-round-pr-create.log').write_text(out);url=out.strip().splitlines()[-1];number=url.rsplit('/',1)[-1];assert number.isdigit();Path('artifacts/cover-long-round-pr-number.json').write_text(json.dumps(dict(number=int(number),url=url)))
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);Path('artifacts/cover-long-round-adoption-ai-usage.json').write_text(usage);p=Path('artifacts/cover-long-round-adoption-comment.md');p.write_text('最終検証を完了し、既定ONで採用します。\n\n'+body+'\nその時点のAI資源量:\n```json\n'+usage+'```\n');out=run(['gh','pr','comment',number,'--repo','iwiwi/mip-codex','--body-file',str(p)]);Path('artifacts/cover-long-round-adoption-comment.log').write_text(out);print(out,flush=True)
v=json.loads(run(['gh','pr','view',number,'--repo','iwiwi/mip-codex','--json','headRefOid,isDraft,state']));assert v['state']=='OPEN'and v['headRefOid']==head
if v['isDraft']:run(['gh','pr','ready',number,'--repo','iwiwi/mip-codex'])
run(['gh','pr','merge',number,'--repo','iwiwi/mip-codex','--squash','--match-head-commit',head]);v=json.loads(run(['gh','pr','view',number,'--repo','iwiwi/mip-codex','--json','headRefOid,state,mergeCommit,mergedAt,url']));assert v['state']=='MERGED'and v['headRefOid']==head;Path('artifacts/cover-long-round-pr-merge.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v),flush=True)
