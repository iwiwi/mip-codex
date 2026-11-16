import json,sys,os
from pathlib import Path
from fractions import Fraction as F
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
root=Path.cwd();os.sched_setaffinity(0,{45});o=root/'artifacts/lp-primal-edge-pricing-root-point-audit';o.mkdir();manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};records=[]
for name,cpu in [('germanrr',41),('neos-860300',44)]:
 m=manifest[name+'.mps.gz'];assert m['split']=='development';model=root/m['path'];assert sha256(model)==m['sha256'];names,variables,rows,cost,offset=exact_model(model,True)
 for label in ['before','after']:
  p=root/f'artifacts/lp-primal-edge-pricing-root-{name}-{cpu}/00'/label/'root-lp.json'
  if not p.exists():continue
  r=json.loads(p.read_text());point={k:F(str(v))for k,v in zip(r['names'],r['point'])};assert set(point)==set(names);bound=F(0);rowv=F(0)
  for k,v in point.items():
   lo,hi=variables[k]['lower'],variables[k]['upper'];bound=max(bound,lo-v if lo is not None else F(0),v-hi if hi is not None else F(0))
  for row in rows:
   value=sum(a*point[k]for k,a in row['coefficients'].items());rowv=max(rowv,row['lower']-value if row['lower']is not None else F(0),value-row['upper']if row['upper']is not None else F(0))
  objective=offset+sum(a*point[k]for k,a in cost.items());error=abs(objective-F(str(r['objective'])));valid=bound<=F(1,1000000)and rowv<=F(1,1000000)and error<=F(1,100000000)*(1+abs(objective));record=dict(name=name,label=label,input_sha256=m['sha256'],dump_sha256=sha256(p),valid=valid,bound_violation=str(bound),row_violation=str(rowv),objective_error=str(error),arithmetic='Fraction from original MPS tokens and saved LP decimal point; integrality intentionally relaxed');records.append(record);atomic_json(o/'records.json',records);print({k:v for k,v in record.items()if k not in ['input_sha256','dump_sha256']},flush=True)
assert len(records)==3;atomic_json(o/'summary.json',dict(records=len(records),valid=sum(r['valid']for r in records)));atomic_json(o/'exit.json',dict(status='complete'if all(r['valid']for r in records)else'invalid'))
