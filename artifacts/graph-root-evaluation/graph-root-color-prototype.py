import sys,json,time,os,itertools
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'));os.sched_setaffinity(0,set(range(33,42)))
from verify_cover_cuts import exact_model
from benchmark_common import sha256
out=root/'artifacts/graph-root-color-prototype';out.mkdir(exist_ok=True)
for name,path in [('sorrell3','data/raw/benchmark/sorrell3.mps.gz'),('sorrell7','data/raw/exploratory/sorrell7.mps.gz')]:
 model=root/path;names,variables,rows,cost,offset=exact_model(model,True);n=len(names);index={name:j for j,name in enumerate(names)};conf=[0]*n
 for row in rows:
  if row['lower']is None and row['upper']is None:continue
  entries=[(index[v],a)for v,a in row['coefficients'].items()if a];lower=row['lower'];upper=row['upper']
  if entries and all(a<0 for _,a in entries):lower,upper=(-upper if upper is not None else None),(-lower if lower is not None else None);entries=[(v,-a)for v,a in entries]
  assert lower is None or lower<=0
  assert all(a>0 for _,a in entries)
  if upper is None:continue
  weights=sorted(a for _,a in entries);assert upper>=max(weights,default=0)
  if len(entries)<2:continue
  assert upper<weights[0]+weights[1]
  for(a,_),(b,_)in itertools.combinations(entries,2):conf[a]|=1<<b;conf[b]|=1<<a
 degrees=[v.bit_count()for v in conf];records=[]
 for mode in ['degree_asc','degree_desc','common_neighbors']:
  started=time.monotonic();remaining=(1<<n)-1;groups=[]
  priority=sorted(range(n),key=lambda j:((degrees[j]if mode=='degree_asc'else-degrees[j]),j));rank={v:k for k,v in enumerate(priority)}
  while remaining:
   available=remaining;group=[]
   while available:
    vertices=[];bits=available
    while bits:bit=bits&-bits;bits^=bit;vertices.append(bit.bit_length()-1)
    if mode=='common_neighbors':v=max(vertices,key=lambda j:((conf[j]&available).bit_count(),-rank[j]))
    else:v=min(vertices,key=lambda j:rank[j])
    group.append(v);remaining&=~(1<<v);available&=conf[v]
   groups.append(group)
  assert sorted(v for g in groups for v in g)==list(range(n))
  assert all(conf[a]&(1<<b)for g in groups for a,b in itertools.combinations(g,2))
  r=dict(mode=mode,colors=len(groups),seconds=time.monotonic()-started);records.append(r);print(name,json.dumps(r),flush=True)
 started=time.monotonic();colors=[-1]*n;saturation=[0]*n;uncolored=set(range(n));groups=[]
 while uncolored:
  v=max(uncolored,key=lambda j:(saturation[j].bit_count(),n-1-degrees[j],-j));used=saturation[v];color=0
  while used&(1<<color):color+=1
  colors[v]=color
  if color==len(groups):groups.append([])
  groups[color].append(v);uncolored.remove(v)
  for u in uncolored:
   if not conf[v]&(1<<u):saturation[u]|=1<<color
 assert all(conf[a]&(1<<b)for g in groups for a,b in itertools.combinations(g,2))
 r=dict(mode='dsatur',colors=len(groups),seconds=time.monotonic()-started);records.append(r);print(name,json.dumps(r),flush=True)
 (out/(name+'.json')).write_text(json.dumps(dict(input_sha256=sha256(model),n=n,records=records),indent=2)+'\n')
