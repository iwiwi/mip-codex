from pathlib import Path
import os,sys,json,subprocess,ast,itertools
from fractions import Fraction as F
root=Path.cwd();os.sched_setaffinity(0,{39});os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
from verify_cover_cuts import exact_model,validate_cut_proofs
from verify_binary_graph import maximize_sense
out=root/'artifacts/pump-limited-mixed-failure-audit';out.mkdir();records=[]
def oracle(model):
 names,vars,rows,cost,offset=exact_model(model,True);ints=[n for n in names if vars[n]['integer']];cont=[n for n in names if not vars[n]['integer']];assert len(cont)==2
 assert all(vars[n]['lower']==0 and vars[n]['upper']==1 for n in ints)
 maxi=maximize_sense(model);best=None;bestpoint=None;assignments=[]
 for bits in itertools.product([0,1],repeat=len(ints)):
  iv=dict(zip(ints,bits));ineq=[];bad=False
  for row in rows:
   a=row['coefficients'].get(cont[0],F(0));b=row['coefficients'].get(cont[1],F(0));fixed=sum(row['coefficients'].get(n,F(0))*v for n,v in iv.items())
   for aa,bb,c in ([(a,b,row['upper']-fixed)]if row['upper']is not None else[])+([(-a,-b,fixed-row['lower'])]if row['lower']is not None else[]):
    if aa==bb==0:
     if c<0:bad=True
    else:ineq.append((aa,bb,c))
  if bad:assignments.append(dict(integer_values=bits,best=None));continue
  for i,n in enumerate(cont):
   lo=vars[n]['lower'];up=vars[n]['upper'];assert lo is not None and up is not None
   ineq.extend([(F(1-i),F(i),up),(-F(1-i),-F(i),-lo)])
  value=None;point=None;count=0
  for (a,b,c),(d,e,f)in itertools.combinations(ineq,2):
   determinant=a*e-d*b
   if determinant==0:continue
   x=(c*e-f*b)/determinant;y=(a*f-d*c)/determinant
   if not all(aa*x+bb*y<=cc for aa,bb,cc in ineq):continue
   count+=1;obj=offset+sum(cost.get(n,F(0))*v for n,v in iv.items())+cost.get(cont[0],F(0))*x+cost.get(cont[1],F(0))*y
   if value is None or(obj>value if maxi else obj<value):value=obj;point=dict(iv,**{cont[0]:x,cont[1]:y})
  assignments.append(dict(integer_values=bits,best=None if value is None else str(value),feasible_vertices=count))
  if value is not None and(best is None or(value>best if maxi else value<best)):best=value;bestpoint=point
 assert best is not None
 return dict(exact_optimum=str(best),float_optimum=float(best),maximize=maxi,point={n:str(v)for n,v in bestpoint.items()},assignments=assignments,method='enumerate every binary assignment and every pairwise boundary intersection of bounded2D continuous polytope; exact Fraction inequalities; no tolerance')
failures=[]
for line in Path('artifacts/pump-limited-mixed-oracle.log').read_text().splitlines():
 try:r=json.loads(line)
 except ValueError:continue
 if r.get('status')=='fail':failures.append((r['case'],ast.literal_eval(r['error'])))
for case,initial in failures:
 model=root/f'artifacts/pump-limited-mixed-oracle/failure-{case}.mps';d=out/str(case);d.mkdir();atomic_json(d/'original-assert-result.json',initial);v=validate(model,initial);atomic_json(d/'original-point-validation.json',v);assert v['valid'];ref=oracle(model);atomic_json(d/'exact-oracle.json',ref)
 row=dict(case=case,input_sha256=sha256(model),exact_optimum=ref['exact_optimum'],original_point_gmp_valid=True,runs=[])
 for label,binary,flag in [('canonical',Path('/home/ubuntu/mip-codex/target/release/mip-codex'),None),('candidate0',root/'artifacts/pump-limited-pilot/frozen/bin/mip-codex','0'),('candidate1',root/'artifacts/pump-limited-pilot/frozen/bin/mip-codex','1')]:
  q=d/label;q.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_BINARY_GRAPH']='0'
  if flag is not None:env['MIP_PUMP_LIMITED_ROUNDING']=flag
  run=subprocess.run([str(binary),'solve',str(model),'10'],capture_output=True,text=True,env=env,timeout=30);(q/'stdout.log').write_text(run.stdout);(q/'stderr.log').write_text(run.stderr);assert run.returncode==0;r=json.loads(run.stdout);atomic_json(q/'result.json',r);v=validate(model,r);atomic_json(q/'validation.json',v);assert v['valid'];p=validate_cut_proofs(model,r);atomic_json(q/'proof.json',p);assert p['valid'];assert r['pump']['calls']==0
  row['runs'].append(dict(label=label,binary_sha256=sha256(binary),status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],original_conflict=r['optimal_original_conflict'],pump_calls=0,gmp_valid=True,proof_valid=True))
 records.append(row);atomic_json(out/'records.json',records);print(json.dumps(row),flush=True)
atomic_json(out/'summary.json',dict(models=5,initial_points_gmp_valid=5,reproductions=15,reproduction_points_gmp_valid=15,reproduction_proof_valid=15,pump_called=False,canonical_statuses=[r['runs'][0]['status']for r in records],false_optimality_claims=sum(x['status']=='optimal'for r in records for x in r['runs']),exact_fraction_oracles=5))
