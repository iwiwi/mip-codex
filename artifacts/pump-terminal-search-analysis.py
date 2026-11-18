from pathlib import Path
import json,sys,collections
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_binary_graph import maximize_sense
out=root/'artifacts/pump-terminal-search-pilot';jobs=json.load(open(out/'jobs.json'));modes=['depth','best','hybrid'];points=0;rows=[];counts={m:collections.Counter()for m in modes[1:]};conditional={m:collections.Counter()for m in modes[1:]}
for i in range(3):assert json.load(open(out/f'worker-{i}-exit.json'))['status']=='complete'
def compare(a,b,dual,sign):
 if dual and(a is None or b is None):return'incomparable'
 if a is None and b is None:return'equal'
 if a is None:return'improved'
 if b is None:return'worsened'
 diff=sign*(b-a)*(1 if dual else -1);tol=1e-8*(1+max(abs(a),abs(b)));return'equal'if abs(diff)<=tol else'improved'if diff>0 else'worsened'
for job in jobs:
 model=root/job['path'];assert sha256(model)==job['sha256'];sign=-1 if maximize_sense(model)else 1;pair={m:json.load(open(out/'cases'/job['name']/m/'result.json'))for m in modes};row=dict(name=job['name'],modes={m:{k:r.get(k)for k in ['status','objective','dual_bound','nodes','seconds','pump','search_best_pops','max_open_nodes']}for m,r in pair.items()});row['all_modes_pump_accepted']=all(r['pump']['improvements']>0 for r in pair.values())
 for m,r in pair.items():
  d=out/'cases'/job['name']/m;assert json.load(open(d/'proof.json'))['valid']
  if r.get('values')is not None:
   points+=1
   for kind in ['strict','official']:assert json.load(open(d/'audit'/(kind+'-validation.json')))['valid']
 for m in modes[1:]:
  for label,field in [('primal','objective'),('dual','dual_bound')]:
   cmp=compare(pair['depth'][field],pair[m][field],label=='dual',sign);counts[m][label+'_'+cmp]+=1;row[m+'_'+label]=cmp
   if row['all_modes_pump_accepted']:conditional[m][label+'_'+cmp]+=1
 rows.append(row)
result=dict(models=17,outputs=51,proof_valid=51,gmp_valid=points,official_valid=points,comparisons=counts,all_modes_pump_accepted_models=[r['name']for r in rows if r['all_modes_pump_accepted']],conditional_comparisons=conditional,rows=rows,adaptive_switch_implemented=False,new_record=False);atomic_json(out/'analysis.json',result);print(json.dumps({k:v for k,v in result.items()if k!='rows'}),flush=True)
for r in rows:
 if r['all_modes_pump_accepted']:print(r['name'],[(m,r['modes'][m]['objective'],r['modes'][m]['dual_bound'],r['modes'][m]['nodes'])for m in modes],flush=True)
