"""保存runを主解の独立検査結果とともに集計。最適化は実行しない。"""
import json, math, subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
phases={
 'child_order':(['smoke-20261115-070635','smoke-20261115-070636'],'rust-pseudocost'),
 'queue':(['smoke-20261115-070950','smoke-20261115-070952'],'rust-depth'),
 'probe':(['smoke-20261115-071541','smoke-20261115-071542'],'rust-no-probe'),
}
out={}
for phase,(runs,reference) in phases.items():
 rows=[]; scores={}
 for run in runs:
  directory=root/'artifacts'/run
  meta=json.loads((directory/'experiment.json').read_text());assert not meta['dirty']
  for name in meta['names']:
   record=manifest[name];assert record['split']=='development'
   model=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'dump',str(root/record['path'])]))
   row=dict(name=name,run=run,source=meta['git_sha'],seconds=meta['seconds'],cpu=meta['cpu'],maximize=model['maximize'],modes={})
   for mode in meta['solvers']:
    p=directory/name/mode
    result=json.loads((p/'result.json').read_text())
    check=json.loads((p/'validation.json').read_text()) if (p/'validation.json').exists() else {}
    row['modes'][mode]={k:result.get(k) for k in ['status','objective','dual_bound','nodes','lp_solves','first_primal_seconds','max_open_nodes','search_best_pops','probe']}
    row['modes'][mode]['primary_valid']=check.get('valid')
   before=row['modes'][reference]
   for mode,after in row['modes'].items():
    if mode==reference:continue
    a=before['objective'] if before['primary_valid'] is True else None
    b=after['objective'] if after['primary_valid'] is True else None
    if a is None: outcome='same' if b is None else 'improved'
    elif b is None:outcome='worse'
    elif math.isclose(a,b,rel_tol=1e-9,abs_tol=1e-7):outcome='same'
    else:outcome='improved' if (b>a if model['maximize'] else b<a) else 'worse'
    after['versus_reference']=outcome
    counts=scores.setdefault(mode,dict(improved=0,worse=0,same=0));counts[outcome]+=1
   rows.append(row)
 out[phase]=dict(reference=reference,score=scores,records=rows)
(root/'artifacts/search-summary.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v['score'] for k,v in out.items()},indent=2))
