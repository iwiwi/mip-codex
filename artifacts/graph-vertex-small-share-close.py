from pathlib import Path
import os,json,time,subprocess,sys,collections
os.sched_setaffinity(0,{38});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json
required=['graph-vertex-small-share-short-controls','graph-vertex-small-share-long-controls','graph-vertex-small-share-connected','graph-vertex-small-share-development-parallel-audit']+['graph-vertex-connected-native/'+s for s in ['highs','scip','glpk']]
start=time.monotonic()
while not all((root/'artifacts'/p/'exit.json').exists()for p in required):
 assert time.monotonic()-start<1800,'close waiter timed out; preserve unfinished logs'
 time.sleep(2)
for name in required:assert json.load(open(root/'artifacts'/name/'exit.json'))['status']=='complete',name
subprocess.run([str(root/'.venv/bin/python'),'artifacts/graph-vertex-small-share-development-analysis.py'],stdout=(root/'artifacts/graph-vertex-small-share-development-analysis.log').open('w'),stderr=subprocess.STDOUT,check=True)
s=(root/'artifacts/graph-vertex-half-long-official.py').read_text().replace('graph-vertex-half-long-official','graph-vertex-small-share-long-official').replace('graph-vertex-long-controls','graph-vertex-small-share-long-controls')
p=root/'artifacts/graph-vertex-small-share-long-official.py';p.write_text(s)
subprocess.run([str(root/'.venv/bin/python'),str(p)],stdout=(root/'artifacts/graph-vertex-small-share-long-official.log').open('w'),stderr=subprocess.STDOUT,check=True)
r=json.load(open(root/'artifacts/graph-vertex-small-share-connected/records.json'));pairs=[];counts=collections.Counter()
for case in range(12):
 off=next(x for x in r if x['case']==case and x['flag']==0);on=next(x for x in r if x['case']==case and x['flag']==1);cmp='equal'if off['objective']==on['objective']else'improved'if on['objective']<off['objective']else'worsened';counts[cmp]+=1;pairs.append(dict(case=case,degree=off['degree'],before=off['objective'],after=on['objective'],before_dual=off['dual_bound'],after_dual=on['dual_bound'],primal=cmp))
atomic_json(root/'artifacts/graph-vertex-small-share-connected-analysis.json',dict(pairs=pairs,counts=counts,all_24_gmp_proof_valid=all(x['point_valid']and x['proof_valid']for x in r),native={s:json.load(open(root/'artifacts/graph-vertex-connected-native'/s/'records.json'))for s in ['highs','scip','glpk']},no_optimality_or_record_claim=True))
atomic_json(root/'artifacts/graph-vertex-small-share-close.json',dict(status='complete',required=required,long_official_points=4,connected_counts=counts))
print(json.dumps(dict(status='complete',connected_counts=counts)),flush=True)
