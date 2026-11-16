from pathlib import Path
from fractions import Fraction as F
import json,sys,os
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{0});model=Path('data/raw/benchmark/neos-3754480-nidda.mps.gz');manifest=next(r for r in json.load(open('data/manifest.json'))['instances']if r['name']==model.name);assert manifest['split']=='development'and sha256(model)==manifest['sha256'];names,variables,rows,cost,offset=exact_model(model,True);indices={n:i for i,n in enumerate(names)};results=[];proofs=[]
for path in sorted(Path('artifacts/numerical-certificate-v2-witnesses').glob('accepted*.json')):
 d=json.load(open(path));assert d['names']==names and d['row_names']==[r['name'] for r in rows]
 lower=[];upper=[]
 for j,n in enumerate(names):
  for side,container in [('lower',lower),('upper',upper)]:
   value=d[side][j];original=variables[n][side]
   if value is None:container.append(None)
   elif original is not None and float(original)==value:container.append(original)
   else:assert variables[n]['integer'] and value==round(value) and abs(value)<=2**52;container.append(F(value))
 found=False
 for sign in [1,-1]:
  w=[sign*F(v) for v in d['weights']];assert len(w)==len(rows)
  a=[F(0) for _ in names];rhs=F(0);valid=True
  for weight,row in zip(w,rows):
   if not weight:continue
   bound=row['upper' if weight>0 else 'lower']
   if bound is None:valid=False;break
   rhs+=weight*bound
   for n,c in row['coefficients'].items():a[indices[n]]+=weight*c
  if not valid:continue
  minimum=F(0)
  for j,c in enumerate(a):
   if not c:continue
   bound=lower[j] if c>0 else upper[j]
   if bound is None:valid=False;break
   minimum+=c*bound
  if valid and minimum>rhs:
   proofs.append(dict(witness=str(path),sha256=sha256(path),node=d['node'],sign=sign,margin=str(minimum-rhs)));found=True;break
 assert found, path
atomic_json(Path('artifacts/numerical-certificate-v2-exact-audit.json'),dict(input_sha256=sha256(model),proofs=proofs,verified=len(proofs),criterion='exact original rows and original node bounds'))
print('exact proofs verified',len(proofs))
