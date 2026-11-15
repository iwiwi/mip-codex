"""保存した比較を集計する。"""
from pathlib import Path
import json, math, subprocess
root=Path(__file__).resolve().parent.parent
manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
phases={'seeded_queue':(['smoke-20261115-072502','smoke-20261115-072504'],'rust-depth'),'covers':(['smoke-20261115-073944','smoke-20261115-073946'],'rust-no-cuts')}
summary={}
for phase,(runs,reference) in phases.items():
 rows=[];counts={}
 for run in runs:
  p=root/'artifacts'/run;meta=json.loads((p/'experiment.json').read_text())
  assert not meta['dirty']
  for name in meta['names']:
   model=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'dump',str(root/manifest[name]['path'])]))
   row=dict(name=name,run=run,source=meta['git_sha'],modes={})
   for mode in meta['solvers']:
    r=json.loads((p/name/mode/'result.json').read_text());check=p/name/mode/'validation.json'
    valid=json.loads(check.read_text())['valid'] if check.exists() else None
    row['modes'][mode]=dict(primal=r['objective'] if valid is True else None,dual=r['dual_bound'],nodes=r['nodes'],lp_solves=r['lp_solves'],cuts=len(r.get('cut_proofs',[])),cut_seconds=r.get('cut_seconds'),status=r['status'])
   for mode,r in row['modes'].items():
    if mode==reference:continue
    a=row['modes'][reference]['primal'];b=r['primal']
    if a is None:outcome='same' if b is None else 'improved'
    elif b is None:outcome='worse'
    elif math.isclose(a,b,rel_tol=1e-9,abs_tol=1e-7):outcome='same'
    else:outcome='improved' if (b>a if model['maximize'] else b<a) else 'worse'
    r['versus_reference']=outcome
    counts.setdefault(mode,dict(improved=0,worse=0,same=0))[outcome]+=1
   rows.append(row)
 summary[phase]=dict(records=rows,score=counts)
records={}
for flag in ['on','off']:
 rs=[]
 for line in (root/'artifacts'/('cover-final-binary-'+flag+'.jsonl')).read_text().splitlines():
  try:r=json.loads(line)
  except ValueError:continue
  if r.get('status')=='pass':rs.append(r)
 assert len(rs)==1000
 records[flag]=rs
 summary['binary_'+flag]=dict(cases=len(rs),nodes=sum(r['nodes'] for r in rs),lp_solves=sum(r['lp_solves'] for r in rs),seconds=sum(r['seconds'] for r in rs),cuts=sum(r['cut_count'] for r in rs),cases_with_cuts=sum(r['cut_count']>0 for r in rs))
assert all(a['sha256']==b['sha256'] for a,b in zip(records['on'],records['off']))
(root/'artifacts/cover-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({k:v.get('score',v) for k,v in summary.items()},indent=2))
