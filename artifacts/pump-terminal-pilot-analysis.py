from pathlib import Path
import json,sys,collections
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_binary_graph import maximize_sense
out=root/'artifacts/pump-terminal-pilot';jobs=json.load(open(out/'jobs.json'));assert len(jobs)==17
for shard in [0,1]:assert json.load(open(out/f'worker-{shard}-exit.json'))['status']=='complete'
counts=collections.Counter();rows=[];points=0;new=[];calls=[]
for job in jobs:
 model=root/job['path'];assert sha256(model)==job['sha256'];sign=-1 if maximize_sense(model)else 1;pair={flag:json.load(open(out/'cases'/job['name']/str(flag)/'result.json'))for flag in [0,1]};row=dict(name=job['name'],before={k:pair[0].get(k)for k in ['status','objective','dual_bound','seconds','pump']},after={k:pair[1].get(k)for k in ['status','objective','dual_bound','seconds','pump']})
 for flag,r in pair.items():
  d=out/'cases'/job['name']/str(flag);assert json.load(open(d/'proof.json'))['valid']
  if r.get('values')is not None:
   points+=1
   for kind in ['strict','official']:assert json.load(open(d/'audit'/(kind+'-validation.json')))['valid']
 for metric,field in [('primal','objective'),('dual','dual_bound')]:
  a=pair[0][field];b=pair[1][field]
  if metric=='dual'and(a is None or b is None):cmp='incomparable'
  elif a is None and b is None:cmp='equal'
  elif a is None:cmp='improved'
  elif b is None:cmp='worsened'
  else:
   difference=sign*(b-a)*(1 if metric=='dual'else -1);tolerance=1e-8*(1+max(abs(a),abs(b)));cmp='equal'if abs(difference)<=tolerance else'improved'if difference>0 else'worsened'
  row[metric]=cmp;counts[metric+'_'+cmp]+=1
 if pair[1]['pump']['terminal_repair_calls']:calls.append(row)
 if pair[1]['pump']['terminal_repair_candidates']:new.append(row)
 rows.append(row)
atomic_json(out/'analysis.json',dict(models=17,outputs=34,gmp_valid=points,official_valid=points,proof_valid=34,comparisons=counts,actual_repair_models=len(calls),new_candidate_models=len(new),new_candidate_rows=new,rows=rows));print(json.dumps(dict(models=17,outputs=34,gmp_valid=points,proof_valid=34,comparisons=counts,actual_repair_models=len(calls),new_candidate_models=len(new))),flush=True)
for row in new:print(row['name'],row['before']['objective'],row['after']['objective'],row['after']['pump'],flush=True)
