from pathlib import Path
import os,sys,json,itertools,collections
os.sched_setaffinity(0,{43});sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import sha256,atomic_json
p=Path('data/raw/exploratory/sorrell8.mps.gz');names,variables,rows,cost,offset=exact_model(p,True);index={n:i for i,n in enumerate(names)};parent=list(range(len(names)))
def find(a):
 while parent[a]!=a:parent[a]=parent[parent[a]];a=parent[a]
 return a
for r in rows:
 if r['lower']is None and r['upper']is None:continue
 js=[index[n]for n,a in r['coefficients'].items()if a]
 for j in js[1:]:parent[find(j)]=find(js[0])
groups=collections.defaultdict(list)
for j in range(len(names)):groups[find(j)].append(j)
res=dict(input_sha256=sha256(p),variables=len(names),components=len(groups),sizes=sorted(map(len,groups.values()),reverse=True),selection='all original finite rows connect every nonzero endpoint; no optimization performed',outside_all_240=True)
atomic_json(Path('artifacts/graph-components-inputs/components.json'),res);print(json.dumps(res))
