"""保存したon/off実験を再集計。最適化は実行しない。"""
from pathlib import Path
import json
import math
root=Path(__file__).resolve().parent
phases={
 'initial':['smoke-20261115-065246','smoke-20261115-065322'],
 'batch':['smoke-20261115-065534','smoke-20261115-065535'],
 'fallback':['smoke-20261115-065807','smoke-20261115-065808'],
}
out={}
for phase,runs in phases.items():
 rows=[]
 for run in runs:
  meta=json.loads((root/run/'experiment.json').read_text())
  assert not meta['dirty']
  for name in meta['names']:
   row=dict(name=name,run=run,source=meta['git_sha'],seconds=meta['seconds'],cpu=meta['cpu'])
   for mode in ['rust-no-dive','rust-dive']:
    directory=root/run/name/mode
    r=json.loads((directory/'result.json').read_text())
    v=json.loads((directory/'validation.json').read_text()) if (directory/'validation.json').exists() else None
    row[mode]={k:r[k] for k in ['status','objective','dual_bound','nodes','lp_solves','first_primal_seconds','diving']}
    row[mode]['primary_valid']=None if v is None else v['valid']
   a=row['rust-no-dive']['objective'];b=row['rust-dive']['objective']
   row['primal_changed']=a!=b and (a is None or b is None or not math.isclose(a,b,rel_tol=1e-9,abs_tol=1e-7))
   rows.append(row)
 out[phase]=dict(records=rows,changed=[r['name'] for r in rows if r['primal_changed']])
(root/'diving-summary.json').write_text(json.dumps(out,indent=2)+'\n')
