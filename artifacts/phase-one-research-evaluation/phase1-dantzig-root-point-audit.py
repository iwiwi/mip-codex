import json,sys,os
from decimal import Decimal
from fractions import Fraction as F
from pathlib import Path
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{37});records=[]
for name,row_flag in [('neos-860300',0),('neos-860300',1),('germanrr',0)]:
 directory=Path('artifacts')/f'phase1-dantzig-root-{name}-row{row_flag}';meta=json.load(open(directory/'experiment.json'));model=Path('data/raw/benchmark')/(name+'.mps.gz');assert sha256(model)==meta['input_sha256'];names,variables,rows,cost,offset=exact_model(model,True)
 for p in sorted(directory.glob('*/root-lp.json')):
  d=json.loads(p.read_text(),parse_float=Decimal);assert names==d['names'];point={n:F(x)for n,x in zip(names,d['point'])};bv=F(0);rv=F(0)
  for n,v in variables.items():
   if v['lower']is not None:bv=max(bv,v['lower']-point[n])
   if v['upper']is not None:bv=max(bv,point[n]-v['upper'])
  for row in rows:
   activity=sum((c*point[n]for n,c in row['coefficients'].items()),F(0))
   if row['lower']is not None:rv=max(rv,row['lower']-activity)
   if row['upper']is not None:rv=max(rv,activity-row['upper'])
  objective=offset+sum((c*point[n]for n,c in cost.items()),F(0));error=abs(objective-F(d['objective']));r=dict(name=name,point=str(p),point_sha256=sha256(p),valid=bv<=F(1,10**6)and rv<=F(1,10**6)and error<=F(1,10**8)*(1+abs(objective)),bounds=str(bv),rows=str(rv),objective_error=str(error),purpose='continuous LP point only, no integer feasibility claim');records.append(r);atomic_json(Path('artifacts/phase1-dantzig-root-point-audit-progress.json'),dict(completed=len(records),valid=sum(r['valid']for r in records)))
atomic_json(Path('artifacts/phase1-dantzig-root-point-audit.json'),dict(checked=len(records),valid=sum(r['valid']for r in records),records=records));print('checked',len(records),'valid',sum(r['valid']for r in records));assert all(r['valid']for r in records)
