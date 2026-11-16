from pathlib import Path
from fractions import Fraction as F
import json,sys,os
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{0});model=Path('data/raw/benchmark/neos-3754480-nidda.mps.gz');manifest=next(r for r in json.load(open('data/manifest.json'))['instances']if r['name']==model.name);assert manifest['split']=='development'and sha256(model)==manifest['sha256'];names,variables,rows,cost,offset=exact_model(model,True);indices={n:i for i,n in enumerate(names)};results=[];proofs=[]
for path in sorted(Path('artifacts/numerical-certificate-v2-witnesses').glob('candidate*.json')):
 d=json.load(open(path));assert d['names']==names and d['row_names']==[r['name']for r in rows];assert d['scaled_columns_total']==d['scaled_rows_total']==0
 if d['multipliers']is None:continue
 lower=[];upper=[]
 for j,n in enumerate(names):
  for side,container in [('lower',lower),('upper',upper)]:
   value=d[side][j];original=variables[n][side]
   if value is None:container.append(None)
   elif original is not None and float(original)==value:container.append(original)
   else:assert variables[n]['integer']and value==round(value)and abs(value)<=2**52;container.append(F(value))
 raw=[F(0)for _ in rows]
 for v,(i,sign)in zip(d['multipliers'],d['row_sources']):raw[i]+=F(v)*F(sign)
 magnitude=max(map(abs,raw));
 if not magnitude:continue
 normalized=[v/magnitude for v in raw]
 for method in ['rational','rounded']:
  for cap in [1,2,3,5,10,16,32,64,128,256,1024,4096,65536,10**6,10**9]:
   weights=[F(float(v)).limit_denominator(cap)if method=='rational'else F(round(float(v)*cap))for v in normalized]
   for sign in [1,-1]:
    w=[sign*v for v in weights];a=[F(0)for _ in names];rhs=F(0);valid=True
    for weight,row in zip(w,rows):
     if not weight:continue
     bound=row['upper'if weight>0 else'lower']
     if bound is None:valid=False;break
     rhs+=weight*bound
     for n,c in row['coefficients'].items():a[indices[n]]+=weight*c
    if not valid:continue
    minimum=F(0);unbounded=0
    for j,c in enumerate(a):
     if not c:continue
     bound=lower[j]if c>0 else upper[j]
     if bound is None:unbounded+=1
     else:minimum+=c*bound
    margin=None if unbounded else minimum-rhs;record=dict(candidate=str(path),node=d['node'],method=method,cap=cap,sign=sign,unbounded_terms=unbounded,margin=None if margin is None else str(margin),valid=margin is not None and margin>0);results.append(record)
    if record['valid']:
     proof=dict(**record,input_sha256=sha256(model),candidate_sha256=sha256(path),weights=[str(v)for v in w],lower=[None if v is None else str(v)for v in lower],upper=[None if v is None else str(v)for v in upper],criterion='exact original-row weighted inequality minimum over the original node box exceeds RHS');proofs.append(proof);print(json.dumps(record),flush=True);break
   if proofs and proofs[-1]['candidate']==str(path):break
  if proofs and proofs[-1]['candidate']==str(path):break
atomic_json(Path('artifacts/numerical-certificate-v2-stalled-search.json'),dict(trials=results,proofs=proofs,verified_contradictions=len(proofs)));print('verified contradictions',len(proofs))
