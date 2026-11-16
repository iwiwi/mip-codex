import random,json,subprocess,struct,itertools
from pathlib import Path
from fractions import Fraction as F
r=random.Random(20261116);out=Path('artifacts/row-ray-bound-synthetic');out.mkdir(exist_ok=True);records=[]
binary=str(Path('target-dual-restoration/release/examples/row_ray_bound').resolve())
for k in range(200):
 n=r.randrange(2,11);maximize=bool(k%2);cost=[r.randrange(-3,10) for _ in range(n)];offset=r.randrange(-10,11);rows=[]
 for i in range(r.randrange(1,12)):
  coeff=[(j,r.randrange(1,6)) for j in range(n) if r.random()<.6]
  if not coeff:coeff=[(0,1)]
  b=r.randrange(1,sum(a for j,a in coeff)+1);rows.append(dict(name=f'r{i}',lower=b,upper=None,coefficients=coeff,exact_integral=True,lower_enclosure=[b,b]))
 model=dict(name=str(k),maximize=maximize,offset=offset,offset_enclosure=[offset,offset],variables=[dict(name=f'x{j}',lower=0,upper=1,integer=True,objective=c,objective_enclosure=[c,c],lower_enclosure=[0,0],upper_enclosure=[1,1]) for j,c in enumerate(cost)],rows=rows)
 p=out/f'{k:03}';p.mkdir(exist_ok=True);(p/'model.json').write_text(json.dumps(model));a=json.loads(subprocess.check_output([binary,str(p/'model.json')],text=True));(p/'result.json').write_text(json.dumps(a))
 sign=-1 if maximize else 1;y=F.from_float(struct.unpack('>d',bytes.fromhex(a['multiplier_bits']))[0]);d=[F(0)]*n;b=0
 for i in a['rows']:
  row=rows[i];b+=row['lower']
  for j,v in row['coefficients']:d[j]+=v
 exact=sign*offset+y*b+sum(min(F(0),sign*cost[j]-y*d[j]) for j in range(n));claimed=F.from_float(struct.unpack('>d',bytes.fromhex(a['normalized_bound_bits']))[0]);assert claimed<=exact
 feasible=[sign*(offset+sum(c*v for c,v in zip(cost,x))) for x in itertools.product([0,1],repeat=n) if all(sum(x[j]*v for j,v in row['coefficients'])>=row['lower'] for row in rows)];assert claimed<=min(feasible)
 records.append(dict(case=k,valid=True,exact_bound=str(exact),optimum=min(feasible),seconds=a['seconds']))
(out/'summary.json').write_text(json.dumps(dict(cases=len(records),valid=sum(a['valid'] for a in records),records=records),indent=2)+'\n');print(len(records),'valid')
