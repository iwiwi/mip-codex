import json,sys,os
from pathlib import Path
from fractions import Fraction as F
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
root=Path.cwd();source=root/'artifacts/scaled-bound-point-repair-oracle';out=root/'artifacts/scaled-bound-point-repair-oracle-point-audit';out.mkdir();os.sched_setaffinity(0,{45});records=[]
for p in sorted(source.glob('*/*/revised-root-lp.json')):
 model=p.parent/'model.mps';names,variables,rows,cost,offset=exact_model(model,True);d=json.loads(p.read_text());point={k:F(str(v))for k,v in zip(d['names'],d['point'])};assert set(point)==set(names);bv=F(0);rv=F(0)
 for k,x in point.items():
  lo,hi=variables[k]['lower'],variables[k]['upper'];bv=max(bv,lo-x if lo is not None else F(0),x-hi if hi is not None else F(0))
 for row in rows:
  a=sum(v*point[k]for k,v in row['coefficients'].items());rv=max(rv,row['lower']-a if row['lower']is not None else F(0),a-row['upper']if row['upper']is not None else F(0))
 objective=offset+sum(v*point[k]for k,v in cost.items());err=abs(objective-F(str(d['objective'])));valid=bv<=F(1,1000000)and rv<=F(1,1000000)and err<=F(1,100000000)*(1+abs(objective));record=dict(case=str(p.parent.relative_to(source)),input_sha256=sha256(model),dump_sha256=sha256(p),valid=valid,bound_violation=str(bv),row_violation=str(rv),objective_error=str(err),integrality='LP relaxation');records.append(record);atomic_json(p.parent/'fraction-point-audit.json',record);atomic_json(out/'summary.json',dict(records=len(records),valid=sum(r['valid']for r in records),invalid=sum(not r['valid']for r in records)))
atomic_json(out/'records.json',records);assert len(records)==500;assert all(r['valid']for r in records);atomic_json(out/'exit.json',dict(status='complete'));print(json.loads((out/'summary.json').read_text()))
