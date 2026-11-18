from pathlib import Path
import json,collections,math,gzip,sys
root=Path.cwd();source=root/'artifacts/pump-limited-development-screen';audit=root/'artifacts/pump-limited-development-parallel-audit';assert json.load(open(audit/'summary.json'))['completed']==376;manifest=[r for r in json.load(open('data/manifest.json'))['instances']if r['split']=='development'];counts=collections.Counter();statuses={x:collections.Counter()for x in ['before','after']};records=[];seconds={x:[]for x in statuses};finite=collections.Counter()
for m in manifest:
 sign=1;pending=False;op=gzip.open if m['path'].endswith('.gz')else open
 with op(root/m['path'],'rt')as f:
  for line in f:
   tokens=line.split()
   if not tokens or tokens[0].startswith('*'):continue
   if tokens[0]=='COLUMNS':break
   if pending:sign=-1 if tokens[0].upper().startswith('MAX')else 1;pending=False
   if tokens[0]=='OBJSENSE':
    if len(tokens)>1:sign=-1 if tokens[1].upper().startswith('MAX')else 1
    else:pending=True
 pair={x:json.load(open(source/'cases'/m['name']/x/'result.json'))for x in statuses};row=dict(name=m['name'],sign=sign)
 for x,r in pair.items():
  a=json.load(open(audit/m['name']/x/'audit-record.json'));assert a['cut_validation']['valid']is True
  if r.get('values')is not None:assert a['strict']['valid']is True and a['official']['valid']is True
  statuses[x][r['status']]+=1;seconds[x].append(min(10,r['seconds'])if r['status']=='optimal'else 10);finite[x]+=int(r['dual_bound']is not None);row[x]={k:r.get(k)for k in ['status','objective','dual_bound','seconds','binary_graph','pump']}
 for metric,field in [('primal','objective'),('dual','dual_bound')]:
  a=pair['before'][field];b=pair['after'][field]
  if metric=='dual'and(a is None or b is None):cmp='incomparable'
  elif a is None and b is None:cmp='equal'
  elif a is None:cmp='improved'
  elif b is None:cmp='worsened'
  else:
   difference=sign*(b-a)*(1 if metric=='dual'else -1);tolerance=1e-8*(1+max(abs(a),abs(b)));cmp='equal'if abs(difference)<=tolerance else'improved'if difference>0 else'worsened'
  counts[metric+'_'+cmp]+=1;row[metric+'_comparison']=cmp
 records.append(row)
result=dict(completed_pairs=len(records),comparison_relative_tolerance=1e-8,comparisons=counts,statuses=statuses,finite_duals=finite,shifted_geometric_seconds_1_unsolved_penalty10={k:math.exp(sum(math.log(t+1)for t in v)/len(v))-1 for k,v in seconds.items()},feature_used=[r for r in records if (r['after'].get('pump')or{}).get('limited_points',0)>0],changed=[r for r in records if any(r[x+'_comparison']in['improved','worsened']for x in['primal','dual'])],records=records,purpose='development only; all points and proof outputs independently checked; inactive-feature bound changes can reflect time cutoffs')
p=root/'artifacts/pump-limited-development-analysis.json';p.write_text(json.dumps(result,indent=2));print(json.dumps({k:v for k,v in result.items()if k not in ['records','changed','feature_used']}));print('feature_used',[(r['name'],r['before']['objective'],r['after']['objective'])for r in result['feature_used']]);print('changed',[(r['name'],r['primal_comparison'],r['dual_comparison'])for r in result['changed']])
