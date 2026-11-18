from pathlib import Path
from fractions import Fraction
import json,hashlib
root=Path.cwd();source=next(Path('/home/ubuntu/mip-numerical-cold-recovery/artifacts/numerical-cold-failure-dumps/neos-2657525-crna.mps.gz').glob('lp.*.json'));saved=json.load(open(source));p=saved['problem'];original=saved['witness'];projected=[0. if not p['equality'][i]and p['slack_upper'][i]is None and y>0 else y for i,y in enumerate(original)]
def check(y):
 f=lambda x:Fraction.from_float(float(x));y=list(map(f,y));lower=sum((a*f(b)for a,b in zip(y,p['rhs'])),Fraction());support=Fraction();unbounded=[]
 for j,col in enumerate(p['columns']):
  a=sum((f(v)*y[i]for i,v in col),Fraction())
  if a>0:
   if p['upper'][j]is None:unbounded.append(['column',j,str(a)])
   else:support+=a*f(p['upper'][j])
 for i,v in enumerate(y):
  if not p['equality'][i]and v>0:
   if p['slack_upper'][i]is None:unbounded.append(['slack',i,str(v)])
   else:support+=v*f(p['slack_upper'][i])
 return dict(valid=not unbounded and lower>support,lower=str(lower),support=str(support),gap=str(lower-support),unbounded_violations=unbounded)
before=check(original);after=check(projected);assert not before['valid']and after['valid'];out=root/'artifacts/farkas-row-projection-independent-check';out.mkdir();(out/'certificate.json').write_text(json.dumps(dict(source=str(source),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),multipliers=projected),indent=2));r=dict(before=before,after=after,changed_rows=[i for i,(a,b)in enumerate(zip(original,projected))if a!=b],arithmetic='exact rational of binary64 values; strict positive contradiction margin; no tolerance',scope='saved node LP only; not original MIP infeasibility',validation_only=True,no_certificate_or_point_transferred_to_Rust=True);(out/'receipt.json').write_text(json.dumps(r,indent=2));print(json.dumps(r))
