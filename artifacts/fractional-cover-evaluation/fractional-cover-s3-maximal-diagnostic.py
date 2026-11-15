import os,json,itertools,time
from pathlib import Path
import highspy
os.sched_setaffinity(0,{2});h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.readModel('data/raw/benchmark/sorrell3.mps.gz');lp=h.getLp();n=lp.num_col_;adj=[set()for _ in range(n)]
matrix=lp.a_matrix_
starts_native,index_native,value_native=matrix.start_,matrix.index_,matrix.value_
assert matrix.format_==highspy.MatrixFormat.kColwise
lower_native,upper_native=lp.row_lower_,lp.row_upper_
rows=[[]for _ in range(lp.num_row_)]
for j in range(n):
 for k in range(starts_native[j],starts_native[j+1]):rows[index_native[k]].append((j,value_native[k]))
for i,row in enumerate(rows):
 assert len(row)==2
 (a,ca),(b,cb)=row;assert a!=b
 assert (ca==cb==1 and upper_native[i]==1 and lower_native[i]<=0)or(ca==cb==-1 and lower_native[i]==-1 and upper_native[i]>=0)
 adj[a].add(b);adj[b].add(a)
cliques=set()
def grow(group):
 candidate=set.intersection(*(adj[v]for v in group)) if group else set(range(n))
 while candidate:
  v=max(candidate,key=lambda v:(len(candidate&adj[v]),len(adj[v]),-v));group.append(v);candidate &=adj[v]
 return tuple(sorted(group))
for v in range(n):cliques.add(grow([v]))
r=json.load(open('artifacts/fractional-cover-enumeration60-pairs/after/sorrell3/result.json'))['binary_graph']
for colors in r['root_prefix_colorings']:
 for color in set(colors):cliques.add(grow([v for v in range(n)if colors[v]==color]))
enumeration_start=time.monotonic();calls=0;stopped=False
def maximal(group,candidate,excluded):
 global calls,stopped
 calls+=1
 if stopped:return
 if len(cliques)>=20000 or time.monotonic()-enumeration_start>=5:stopped=True;return
 if not candidate and not excluded:
  cliques.add(tuple(sorted(group)));return
 pivot=max(candidate|excluded,key=lambda v:len(candidate&adj[v]),default=None)
 choices=sorted(candidate-(adj[pivot]if pivot is not None else set()))
 for v in choices:
  maximal(group+[v],candidate&adj[v],excluded&adj[v]);candidate.remove(v);excluded.add(v)
  if stopped:return
for v in range(n):
 maximal([v],{u for u in adj[v]if u>v},{u for u in adj[v]if u<v})
 if stopped:break
print('maximal enumeration',time.monotonic()-enumeration_start,'calls',calls,'partial',stopped,flush=True)
cliques=sorted(cliques);print('vertices',n,'edges',sum(map(len,adj))//2,'cliques',len(cliques),'sizes',min(map(len,cliques)),max(map(len,cliques)),flush=True)
p=highspy.HighsLp();p.num_col_=len(cliques);p.num_row_=n;p.col_cost_=[1.]*len(cliques);p.col_lower_=[0.]*len(cliques);p.col_upper_=[highspy.kHighsInf]*len(cliques);p.row_lower_=[1.]*n;p.row_upper_=[highspy.kHighsInf]*n;p.a_matrix_.format_=highspy.MatrixFormat.kColwise;starts=[0];index=[]
for c in cliques:index.extend(c);starts.append(len(index))
p.a_matrix_.start_=starts;p.a_matrix_.index_=index;p.a_matrix_.value_=[1.]*len(index);s=highspy.Highs();s.setOptionValue('output_flag',False);s.setOptionValue('threads',1);s.setOptionValue('time_limit',30);s.passModel(p);start=time.monotonic();s.run();sol=s.getSolution();result=dict(purpose='comparison diagnostic only; never used by Rust optimizer',source='sorrell3 development; diagnostic only',highs_version=s.version(),status=str(s.getModelStatus()),seconds=time.monotonic()-start,objective=s.getObjectiveValue(),cliques=cliques,weights=sol.col_value if sol.value_valid else None)
Path('artifacts/fractional-cover-s3-maximal-diagnostic.json').write_text(json.dumps(result)+'\n');print(result['status'],result['objective'],result['seconds'],flush=True)
