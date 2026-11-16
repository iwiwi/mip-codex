"""Saved development LP points only; exact original-row conflict witnesses."""
import argparse,itertools,json,math,os,sys,time
from fractions import Fraction as F
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
p=argparse.ArgumentParser();p.add_argument('--partition',type=int,default=0);p.add_argument('--partitions',type=int,default=4);p.add_argument('--cpu',type=int,default=4);args=p.parse_args();os.sched_setaffinity(0,{args.cpu})
root=Path.cwd();out=root/'artifacts/clique-cut-diagnostic';out.mkdir(exist_ok=True);folders=sorted(d.parent for d in (root/'artifacts/development-root-survey-20261115').glob('*/root-lp.json'));assert len(folders)==77
records=[]
for case,d in enumerate(folders):
 if case%args.partitions!=args.partition:continue
 begin=time.monotonic();meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];data=json.loads((d/'root-lp.json').read_text());names,variables,rows,_,_=exact_model(model,True);assert names==data['names'];index={v:j for j,v in enumerate(names)};x=[F(str(v))for v in data['point']]
 binary={v for v in names if variables[v]['integer']and variables[v]['lower']is not None and variables[v]['upper']is not None and 0<=math.ceil(variables[v]['lower'])<=math.floor(variables[v]['upper'])<=1}
 literals=[]
 for v in binary:
  j=index[v]
  if F(1,10000)<x[j]<F(9999,10000):
   literals.extend([(j,False),(j,True)])
 def value(l):return 1-x[l[0]]if l[1]else x[l[0]]
 literals=sorted(literals,key=lambda l:(-value(l),l))[:512];selected=set(literals);adj={l:set()for l in literals};witness={};projected={};work=0;eligible=0
 for ri,row in enumerate(rows):
  if work>=2_000_000:break
  for upper in [True,False]:
   capacity=row['upper'if upper else'lower'];work+=len(row['coefficients'])
   if capacity is None:continue
   sign=1 if upper else-1;capacity*=sign;items=[];valid=True
   for v,orig in row['coefficients'].items():
    a=sign*orig
    if not a:continue
    if v in binary:
     if a<0:capacity-=a
     l=(index[v],a<0)
     if l in selected:items.append((l,abs(a)))
    else:
     bound=variables[v]['lower'if a>0 else'upper']
     if bound is None:valid=False;break
     capacity-=a*bound
   if not valid or len(items)<2:continue
   eligible+=1;projected[ri,upper]=(dict(items),capacity)
   for (u,a),(v,b)in itertools.combinations(items,2):
    work+=1
    if work>=2_000_000:break
    if a+b<=capacity:continue
    key=tuple(sorted((u,v)))
    if key not in witness:witness[key]=(ri,upper);adj[u].add(v);adj[v].add(u)
   if work>=2_000_000:break
 cliques={}
 for seed in literals:
  clique=[seed];candidates=adj[seed].copy()
  while candidates:
   v=max(candidates,key=lambda v:(value(v),len(adj[v]&candidates),tuple(-int(a)for a in v)))
   clique.append(v);candidates&=adj[v];candidates={u for u in candidates if u[0]!=v[0]}
  key=tuple(sorted(clique));violation=sum(map(value,key),F(0))-1
  if len(key)>=3 and violation>F(1,100000):cliques[key]=violation
 cuts=[]
 for clique,violation in sorted(cliques.items(),key=lambda t:(-t[1],t[0]))[:32]:
  assert len({v for v,neg in clique})==len(clique)
  proofs=[]
  for u,v in itertools.combinations(clique,2):
   ri,upper=witness[tuple(sorted((u,v)))];items,capacity=projected[ri,upper];assert items[u]+items[v]>capacity
   proofs.append(dict(source_row=ri,source_name=rows[ri]['name'],upper_side=upper,literals=[list(u),list(v)],weights=[str(items[u]),str(items[v])],capacity=str(capacity)))
  coeff=[[j,-1 if neg else 1]for j,neg in clique];rhs=1-sum(neg for j,neg in clique);assert sum(a*x[j]for j,a in coeff)-rhs==violation
  cuts.append(dict(coefficients=coeff,upper=rhs,violation=str(violation),pair_proofs=proofs))
 record=dict(name=d.name,split='development',model=str(model),input_sha256=sha256(model),root_dump=str(d/'root-lp.json'),root_dump_sha256=sha256(d/'root-lp.json'),purpose='diagnostic exact original-row conflicts, no Rust changes',literals=len(literals),edges=len(witness),eligible_sides=eligible,work=work,cuts=cuts,seconds=time.monotonic()-begin)
 atomic_json(out/(d.name+'.json'),record);records.append(dict(name=d.name,cuts=len(cuts),max_violation=None if not cuts else cuts[0]['violation'],edges=len(witness),seconds=record['seconds']));print(json.dumps(records[-1]),flush=True)
atomic_json(out/f'partition-{args.partition}.json',dict(records=records,models=len(records),models_with_cuts=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records)))
