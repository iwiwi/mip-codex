import json,sys
from pathlib import Path
from fractions import Fraction as F
from decimal import Decimal
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
model=Path('data/raw/benchmark/neos-3754480-nidda.mps.gz');names,variables,rows,cost,offset=exact_model(model,True);p=Path('artifacts/stalled-node-isolated/93-revised-propagation.json');d=json.loads(p.read_text(),parse_float=Decimal);point={n:F(x)for n,x in d['values'].items()};node=json.load(open(next(Path('artifacts/numerical-certificate-v2-witnesses').glob('candidate.*.93.json'))));bv=F(0);rv=F(0)
for j,n in enumerate(names):
 v=variables[n];x=point[n]
 for key in ['lower','upper']:
  b=node[key][j]
  if b is None:continue
  original=v[key];b=original if original is not None and float(original)==b else F(b)
  bv=max(bv,b-x if key=='lower'else x-b)
for row in rows:
 a=sum((c*point[n]for n,c in row['coefficients'].items()),F(0))
 if row['lower']is not None:rv=max(rv,row['lower']-a)
 if row['upper']is not None:rv=max(rv,a-row['upper'])
obj=offset+sum((c*point[n]for n,c in cost.items()),F(0));oe=abs(obj-F(d['objective']));r=dict(purpose='continuous LP point only; integer feasibility not claimed',valid=bv<=F(1,10**6)and rv<=F(1,10**6)and oe<=F(1,10**8)*(1+abs(obj)),bounds=str(bv),rows=str(rv),objective_error=str(oe),input_sha256=sha256(model),point_sha256=sha256(p));atomic_json(Path('artifacts/stalled-node-point-audit.json'),r);print(r);assert r['valid']
