import os,sys,json,math,time
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import sha256,atomic_json
os.sched_setaffinity(0,{40});root=Path.cwd();manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for name in ['neos-3754480-nidda.mps.gz','app1-1.mps.gz','glass4.mps.gz','leo1.mps.gz','neos-2657525-crna.mps.gz','timtab1.mps.gz','binkar10_1.mps.gz','sp150x300d.mps.gz']:
 r=manifest[name];assert r['split']=='development';p=root/r['path'];assert sha256(p)==r['sha256'];t=time.monotonic();names,variables,rows,cost,offset=exact_model(p,True);counts=dict(rows=len(rows),integer=0,decimal_scalable=0,scalable_with_original_integer_bounds=0);scales={}
 for row in rows:
  values=[*row['coefficients'].values(),*[row[b]for b in ['lower','upper']if row[b]is not None]];d=1
  for v in values:d=math.lcm(d,v.denominator)
  if d==1:counts['integer']+=1;continue
  scale=next((10**k for k in range(1,10)if (10**k)%d==0),None)
  if scale is None or any(abs(v*scale)>2**52 for v in values):continue
  counts['decimal_scalable']+=1;scales[str(scale)]=scales.get(str(scale),0)+1
  if any(variables[n]['integer']for n in row['coefficients'])and all(any(variables[n][b]is not None and variables[n][b].denominator==1 for b in ['lower','upper'])for n in row['coefficients']):counts['scalable_with_original_integer_bounds']+=1
 record=dict(name=name,input_sha256=r['sha256'],seconds=time.monotonic()-t,counts=counts,scales=scales,purpose='exact row eligibility only; no LP/MIP solve or performance claim');records.append(record);print(json.dumps(record),flush=True);atomic_json(root/'artifacts/decimal-row-mir-diagnostic.json',records)
