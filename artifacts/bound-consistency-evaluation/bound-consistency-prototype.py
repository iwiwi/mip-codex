"""Read-only weak-duality diagnostics across independently checked serialized points."""
import argparse,json,sys
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
sys.path.insert(0,str(Path.cwd()/'scripts'))
from benchmark_common import sha256,atomic_json,utc
from verify_binary_graph import maximize_sense
p=argparse.ArgumentParser();p.add_argument('runs',nargs='+',type=Path);p.add_argument('--output',required=True,type=Path);a=p.parse_args()
tolerance=Fraction(1,1000000);groups={};sources=[];warnings=[]
for run in a.runs:
 config=json.loads((run/'config.json').read_text());models={r['name']:r for r in config['instances']};path=run/'results.jsonl';payload=path.read_bytes();sources.append(dict(run=str(run.resolve()),results_sha256=sha256(path),config_sha256=sha256(run/'config.json')))
 for index,line in enumerate(payload.splitlines()):
  try:s=json.loads(line,parse_float=Decimal)
  except json.JSONDecodeError:
   warnings.append(dict(run=str(run),line=index+1,error='incomplete_or_invalid_json_record'));continue
  model=models[s['name']];key=(s['name'],s['input_sha256']);m=Path(model['absolute_path']);assert sha256(m)==key[1]
  sense=maximize_sense(m);g=groups.setdefault(key,dict(maximize=sense,records=[]));assert g['maximize']==sense
  result=run/'cases'/s['job_id']/'result.json'
  if not result.exists():warnings.append(dict(run=str(run),job=s['job_id'],error='missing_result'));continue
  assert sha256(result)==s.get('result_sha256'),str(result)
  r=json.loads(result.read_text(),parse_float=Decimal)
  def number(v):
   if type(v)is int:return Fraction(v)
   if isinstance(v,Decimal)and v.is_finite():return Fraction(v)
   return None
  valid=s.get('strict_validation',{}).get('valid')is True and s.get('primal_bound')is not None
  g['records'].append(dict(source=str(result.resolve()),solver=s['solver'],status=s.get('solver_status'),primal=number(r.get('objective'))if valid else None,dual=number(s.get('dual_bound'))))
findings=[];comparisons=0
for (name,hash_),g in groups.items():
 sign=-1 if g['maximize']else 1;valid=[r for r in g['records']if r['primal']is not None]
 if not valid:continue
 best=min(valid,key=lambda r:sign*r['primal'])
 for r in g['records']:
  if r['dual']is not None:
   comparisons+=1;excess=sign*(r['dual']-best['primal'])
   if excess>tolerance:findings.append(dict(type='reported_dual_exceeds_verified_point',name=name,input_sha256=hash_,maximize=g['maximize'],bound_source=r['source'],bound_solver=r['solver'],reported_dual=str(r['dual']),point_source=best['source'],point_solver=best['solver'],verified_reported_objective=str(best['primal']),absolute_excess=str(excess)))
  if str(r['status']).lower()=='infeasible':findings.append(dict(type='infeasibility_claim_with_verified_point',name=name,input_sha256=hash_,claim_source=r['source'],point_source=best['source']))
atomic_json(a.output,dict(updated_utc=utc(),status='diagnostic_not_a_dual_certificate',absolute_tolerance=str(tolerance),input_groups=len(groups),bound_comparisons=comparisons,findings=findings,warnings=warnings,sources=sources,limitation='Checks weak duality against available verified points, not correctness of every bound or replay of optimality proofs.'))
print(json.dumps(dict(groups=len(groups),comparisons=comparisons,findings=len(findings),warnings=len(warnings))))
