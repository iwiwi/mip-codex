import json,sys,os
from decimal import Decimal
from fractions import Fraction as F
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{38});records=[]
for directory in sorted(Path('artifacts/restoration-progress-selected').iterdir()):
 meta=json.loads((directory/'experiment.json').read_text());model=Path('data/raw/benchmark')/meta['name'];assert sha256(model)==meta['input_sha256'];names,variables,rows,cost,offset=exact_model(model,True)
 for variant in ['before','after']:
  p=directory/variant/'root-lp.json'
  if not p.exists():continue
  d=json.loads(p.read_text(),parse_float=Decimal);assert names==d['names'];point={n:F(x)for n,x in zip(names,d['point'])};bv=F(0);rv=F(0)
  for n,v in variables.items():
   if v['lower']is not None:bv=max(bv,v['lower']-point[n])
   if v['upper']is not None:bv=max(bv,point[n]-v['upper'])
  for row in rows:
   a=sum((c*point[n]for n,c in row['coefficients'].items()),F(0))
   if row['lower']is not None:rv=max(rv,row['lower']-a)
   if row['upper']is not None:rv=max(rv,a-row['upper'])
  obj=offset+sum((a*point[n]for n,a in cost.items()),F(0));oe=abs(obj-F(d['objective']));r=dict(name=meta['name'],variant=variant,valid=bv<=F(1,10**6)and rv<=F(1,10**6)and oe<=F(1,10**8)*(1+abs(obj)),bounds=str(bv),rows=str(rv),objective_error=str(oe),input_sha256=sha256(model),point_sha256=sha256(p),purpose='LP point only, not integer feasibility');records.append(r);print(json.dumps(r),flush=True)
atomic_json(Path('artifacts/restoration-progress-point-audit-preliminary.json'),dict(records=records,checked=len(records),valid=sum(r['valid']for r in records)));assert all(r['valid']for r in records)
