from pathlib import Path
from fractions import Fraction as F
import json,math
out=[]
for p in sorted(Path('artifacts/ratio-residual-dumps').glob('*.json')):
 d=json.load(open(p));rhs=d['rhs'];x=d['point'];rows=[[]for _ in rhs];basis=set(d['basis'])
 for j,col in enumerate(d['columns']):
  for i,a in col:rows[i].append((j,a))
 residual=[F.from_float(b)-sum((F.from_float(a)*F.from_float(x[j])for j,a in row),F(0))for b,row in zip(rhs,rows)]
 maximum=max(map(abs,residual));bad=sorted(range(len(rhs)),key=lambda i:abs(residual[i]),reverse=True)[:5]
 naive=[b-sum(a*x[j]for j,a in row)for b,row in zip(rhs,rows)]
 item=dict(source=str(p),rows=len(rhs),variables=len(x),exact_residual_inf=float(maximum),exact_residual_inf_rational=str(maximum),top_residual_rows=[dict(row=i,exact=float(residual[i]),naive=naive[i])for i in bad],bad_variables=d['bad']);out.append(item);print(json.dumps(item),flush=True)
Path('artifacts/ratio-residual-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
