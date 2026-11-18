from pathlib import Path
import json,subprocess
root=Path.cwd()
def run(args):
 p=subprocess.run(args,capture_output=True,text=True)
 if p.returncode:print(p.stdout,p.stderr,flush=True)
 p.check_returncode();return p.stdout
assert not run(['git','diff','b3da9b9d10','--','src','Cargo.toml','Cargo.lock','build.rs'])
a=json.load(open('artifacts/pump-terminal-default-development-parallel-audit/summary.json'));assert a['completed']==376 and a['cut_valid']==376 and a['strict_valid']==a['official_valid']==133 and a['strict_invalid']==0
r=json.load(open('artifacts/pump-terminal-default-development-analysis.json'));assert r['comparisons']==dict(primal_equal=184,primal_improved=4,dual_equal=173,dual_worsened=2,dual_incomparable=13)
for p in Path('artifacts/pump-terminal-default-development-screen/cases').glob('*/*/job.json'):
 j=json.load(open(p));assert j['split']=='development' and j['threads']==1 and j['memory_bytes']==16*1024**3;assert j['environment'].get('MIP_PUMP_TERMINAL_REPAIR')==('0'if p.parent.name=='before'else None)
run(['python3','artifacts/pump-terminal-archive-preflight.py']);head=run(['git','rev-parse','HEAD']).strip();assert run(['git','ls-remote','origin','refs/heads/pump-terminal-repair']).split()[0]==head
body="""pump補助LPの打切り点が単純丸めで不合格だった場合に、既存のRust座標修復を短く試して可行主解を補います。元変数写像・root変数境界・元行candidate_checkを維持し、補助LPのdual/basis/最適性は使いません。既定ON、MIP_PUMP_TERMINAL_REPAIR=0で従来動作。局所deadlineは残時間1%と100msの小さい方、search work予算500000。初期全行scan等でwork予算を超える消費も生値で保存します。全体期限を過ぎた修復結果は採用せず、新処理は既存全体時間を消費します。LP予算・制約・許容差・枝刈り規則は変えません。

初期3f065d/FP8f433と既定ON b3da9b/FP93bddを別凍結。Rust/Cargo/build差は既定boolだけ。各360 Rust passed/4ignored、各binary200成功。新3テストで拒否丸めからの回収、元node bounds、期限/非有限入力の拒否と親LPの不変を確認。既定ONはCLI SHAcec43d8c410b91f3d4a801dfd7d1304dff7f40f969c2542c029d68b2cb26cf85。

開発188問×OFF/ON、各10秒/16GiB/1thread。初期・既定ONとも各376fresh proof/133点fresh GMP・公式valid。初期はprimal183同等/4改善/1悪化、dual173同等/1改善/1悪化/13比較不能。既定ON・環境変数未指定ではprimal184同等/4改善/0悪化、dual173同等/0改善/2悪化/13比較不能。両182time_limit/2optimal/4numerical_error、有限下界175。既存の保守的数値終了を成功扱いしません。

修復による早期可行点を3問で再現しました。nativeも同マシン10秒/16GiB/1thread、全6点GMP validです。

| 問題 | OFF→修復ON primal | HiGHS1.16.0 | SCIP10.0.2 | GLPK5 |
|---|---:|---:|---:|---:|
| drayage-25-23 | None→645963.170929 | 101282.647018（1.475秒で最適） | 136525.137978 | 812730.223160 |
| graph20-20-1rand | None→-7 | -8 | -9（4.980秒で最適） | -5 |
| h80x6320d | None→26428.164941（再比較） | 未追加測定 | 未追加測定 | 未追加測定 |

本候補は2問でGLPKより主解が良いもののHiGHS/SCIPより劣り、全体優位・新記録は未達です。GLPKの実時間10.196/10.243秒も保持します。

初期pilot17問ではh80が11228.774227→26428.164941へ悪化しました。追加3pairsはOFF全None/ON全26428.164941（6proof/3GMP）でしたが元損失を置換しません。初期188のneos-911970 129.38→140.59と追加1pair111.47→140.59の損失、fast0507の下界悪化、最終seymour1/sp97arの下界悪化も全保持。これらは新処理未実行です。最終mcsched312590→309595の変動もterminal修復未実行で成果に帰属しません。

診断の目的降下は全6点GMP公式validでh80だけ25594.59499へ小改善、本体へ追加しません。探索順序17問51出力と188問376出力も全proof/19点・133点GMP公式valid。global bestは主解4改善/13悪化、dual44改善となり既定化を棄却。動的切換も追加しません。失敗した案も検証済archiveで保持します。

7archiveの全payload SHAと除外ELFを元の物理保存先で再確認。holdout52を判断に使わず、正式240問は未起動です。主解発見の補完を根拠に本候補を採用します。
"""
p=Path('artifacts/pump-terminal-pr-body.md');p.write_text(body);out=run(['gh','pr','create','--repo','iwiwi/mip-codex','--base','main','--head','pump-terminal-repair','--draft','--title','pump打切り点を元行で短く修復して早期主解を補う','--body-file',str(p)]);Path('artifacts/pump-terminal-pr-create.log').write_text(out);url=out.strip().splitlines()[-1];number=url.rsplit('/',1)[-1];assert number.isdigit();Path('artifacts/pump-terminal-pr-number.json').write_text(json.dumps(dict(number=int(number),url=url)))
comment='最終検証を完了し、既定ONで採用します。\n\n'+body
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);Path('artifacts/pump-terminal-adoption-ai-usage.json').write_text(usage);p=Path('artifacts/pump-terminal-adoption-comment.md');p.write_text(comment+'\nその時点のAI資源量:\n```json\n'+usage+'```\n');out=run(['gh','pr','comment',number,'--repo','iwiwi/mip-codex','--body-file',str(p)]);Path('artifacts/pump-terminal-adoption-comment.log').write_text(out);print(out,flush=True)
v=json.loads(run(['gh','pr','view',number,'--repo','iwiwi/mip-codex','--json','headRefOid,isDraft,state']));assert v['state']=='OPEN'and v['headRefOid']==head
if v['isDraft']:run(['gh','pr','ready',number,'--repo','iwiwi/mip-codex'])
run(['gh','pr','merge',number,'--repo','iwiwi/mip-codex','--squash','--match-head-commit',head]);v=json.loads(run(['gh','pr','view',number,'--repo','iwiwi/mip-codex','--json','headRefOid,state,mergeCommit,mergedAt,url']));assert v['state']=='MERGED'and v['headRefOid']==head;Path('artifacts/pump-terminal-pr-merge.json').write_text(json.dumps(v,indent=2)+'\n');print(json.dumps(v),flush=True)
