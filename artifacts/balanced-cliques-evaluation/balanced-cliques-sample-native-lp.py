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
cliques=json.load(open('artifacts/balanced-cliques-sample-candidates.json'))['groups']
assert all(len(c)==len(set(c))and all(b in adj[a]for a,b in itertools.combinations(c,2))for c in cliques)
print('checked candidate cliques',len(cliques),'sizes',min(map(len,cliques)),max(map(len,cliques)),flush=True)
p=highspy.HighsLp();p.num_col_=len(cliques);p.num_row_=n;p.col_cost_=[1.]*len(cliques);p.col_lower_=[0.]*len(cliques);p.col_upper_=[highspy.kHighsInf]*len(cliques);p.row_lower_=[1.]*n;p.row_upper_=[highspy.kHighsInf]*n;p.a_matrix_.format_=highspy.MatrixFormat.kColwise;starts=[0];index=[]
for c in cliques:index.extend(c);starts.append(len(index))
p.a_matrix_.start_=starts;p.a_matrix_.index_=index;p.a_matrix_.value_=[1.]*len(index);s=highspy.Highs();s.setOptionValue('output_flag',False);s.setOptionValue('threads',1);s.setOptionValue('time_limit',30);s.passModel(p);start=time.monotonic();s.run();sol=s.getSolution();result=dict(purpose='comparison diagnostic only; never used by Rust optimizer',source='sorrell3 development; dumped Rust candidates; comparison only',highs_version=s.version(),status=str(s.getModelStatus()),seconds=time.monotonic()-start,objective=s.getObjectiveValue(),cliques=cliques,weights=sol.col_value if sol.value_valid else None)
Path('artifacts/balanced-cliques-sample-native-lp.json').write_text(json.dumps(result)+'\n');print(result['status'],result['objective'],result['seconds'],flush=True)
