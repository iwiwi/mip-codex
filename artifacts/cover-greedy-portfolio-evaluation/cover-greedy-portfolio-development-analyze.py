import json,math,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json
from verify_binary_graph import maximize_sense
root=Path.cwd();source=root/'artifacts/cover-greedy-portfolio-development-screen';audit=root/'artifacts/cover-greedy-portfolio-development-audit';audits={(r['name'],r['variant']):r for r in json.loads((audit/'records.json').read_text())};manifest=[r for r in json.loads((root/'data/manifest.json').read_text())['instances']if r['split']=='development'];records=[];counts={k:0 for k in ['primal_improved','primal_worsened','primal_equal','primal_pending','dual_improved','dual_worsened','dual_equal','dual_incomparable']};optimal={'before':0,'after':0};times={'before':[],'after':[]};statuses={'before':{},'after':{}}
for m in manifest:
 ps={v:source/'cases'/m['name']/v/'summary.json'for v in ['before','after']}
 if not all(p.exists()for p in ps.values()):continue
 data={v:json.loads(p.read_text())for v,p in ps.items()};sign=-1 if maximize_sense(root/m['path'])else 1;r=dict(name=m['name'],before=data['before'],after=data['after'],sign=sign)
 valid={};pending=False
 for v,d in data.items():
  statuses[v][d['status']]=statuses[v].get(d['status'],0)+1;a=audits.get((m['name'],v));valid[v]=a is not None and a.get('strict')is not None and a['strict']['valid']is True
  if d.get('objective')is not None and not valid[v]:pending=True
  solved=d['status']=='optimal'and valid[v];optimal[v]+=solved;times[v].append(min(10,max(0,d.get('seconds',10)))if solved else 10)
 a,b=data['before'].get('objective'),data['after'].get('objective');tol=lambda x,y:1e-8*(1+max(abs(x),abs(y)))
 if pending:r['primal_comparison']='pending'
 elif a is None and b is None:r['primal_comparison']='equal'
 elif a is None:r['primal_comparison']='improved'
 elif b is None:r['primal_comparison']='worsened'
 else:
  delta=sign*(b-a);r['primal_comparison']='improved'if delta < -tol(a,b)else'worsened'if delta>tol(a,b)else'equal'
 counts['primal_'+r['primal_comparison']]+=1
 a,b=data['before'].get('dual_bound'),data['after'].get('dual_bound')
 if a is None or b is None:r['dual_comparison']='incomparable'
 else:
  delta=sign*(b-a);r['dual_comparison']='improved'if delta>tol(a,b)else'worsened'if delta < -tol(a,b)else'equal'
 counts['dual_'+r['dual_comparison']]+=1;records.append(r)
summary=dict(purpose='development-only partial screen; not final benchmark; point validation required for primal counts',completed_pairs=len(records),comparisons=counts,reported_optimal_with_validated_points=optimal,statuses=statuses,shifted_geometric_seconds_1_unsolved_penalty10={v:math.exp(sum(math.log1p(t)for t in ts)/len(ts))-1 if ts else None for v,ts in times.items()},comparison_relative_tolerance=1e-8,records=records)
summary['dual_availability']=dict(before=sum(r['before'].get('dual_bound')is not None for r in records),after=sum(r['after'].get('dual_bound')is not None for r in records),newly_available=sum(r['before'].get('dual_bound')is None and r['after'].get('dual_bound')is not None for r in records),lost=sum(r['before'].get('dual_bound')is not None and r['after'].get('dual_bound')is None for r in records))
atomic_json(root/'artifacts/cover-greedy-portfolio-development-analysis.json',summary);print(json.dumps({k:v for k,v in summary.items()if k!='records'},ensure_ascii=False));print(json.dumps([dict(name=r['name'],primal=r['primal_comparison'],dual=r['dual_comparison'])for r in records if r['primal_comparison']in ['improved','worsened']or r['dual_comparison']in ['improved','worsened']],ensure_ascii=False))
