import os,json,itertools,time
from pathlib import Path
import highspy
os.sched_setaffinity(0,{6});h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.readModel('data/raw/exploratory/sorrell7.mps.gz');lp=h.getLp();n=lp.num_col_;adj=[set()for _ in range(n)]
for i in range(lp.num_row_):
 status,index,values=h.getRowEntries(i)
 assert len(index)==2
 a,b=map(int,index);adj[a].add(b);adj[b].add(a)
cliques=set()
def grow(group):
 candidate=set.intersection(*(adj[v]for v in group)) if group else set(range(n))
 while candidate:
  v=max(candidate,key=lambda v:(len(candidate&adj[v]),len(adj[v]),-v));group.append(v);candidate &=adj[v]
 return tuple(sorted(group))
for v in range(n):cliques.add(grow([v]))
r=json.load(open('artifacts/graph-cover-long-pairs/after/sorrell7/result.json'))['binary_graph']
for colors in r['root_prefix_colorings']:
 for color in set(colors):cliques.add(grow([v for v in range(n)if colors[v]==color]))
cliques=sorted(cliques);print('vertices',n,'edges',sum(map(len,adj))//2,'cliques',len(cliques),'sizes',min(map(len,cliques)),max(map(len,cliques)),flush=True)
p=highspy.HighsLp();p.num_col_=len(cliques);p.num_row_=n;p.col_cost_=[1.]*len(cliques);p.col_lower_=[0.]*len(cliques);p.col_upper_=[highspy.kHighsInf]*len(cliques);p.row_lower_=[1.]*n;p.row_upper_=[highspy.kHighsInf]*n;p.a_matrix_.format_=highspy.MatrixFormat.kColwise;starts=[0];index=[]
for c in cliques:index.extend(c);starts.append(len(index))
p.a_matrix_.start_=starts;p.a_matrix_.index_=index;p.a_matrix_.value_=[1.]*len(index);s=highspy.Highs();s.setOptionValue('output_flag',False);s.setOptionValue('threads',1);s.setOptionValue('time_limit',30);s.passModel(p);start=time.monotonic();s.run();sol=s.getSolution();result=dict(purpose='comparison diagnostic only; never used by Rust optimizer',source='sorrell7 exploration outside benchmark v2',highs_version=s.version(),status=str(s.getModelStatus()),seconds=time.monotonic()-start,objective=s.getObjectiveValue(),cliques=cliques,weights=sol.col_value if sol.value_valid else None)
Path('artifacts/fractional-cover-initial.json').write_text(json.dumps(result)+'\n');print(result['status'],result['objective'],result['seconds'],flush=True)
