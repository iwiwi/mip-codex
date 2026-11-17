from pathlib import Path
from fractions import Fraction as F
from importlib.util import spec_from_file_location,module_from_spec
import json,random,itertools,subprocess,signal,resource,os,sys,time
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/safe-gmi-report-mip-oracle';out.mkdir();os.sched_setaffinity(0,{42});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=root/'artifacts/safe-gmi-report-diagnostic/bin/mip-codex';spec=spec_from_file_location('checker',root/'artifacts/safe-gmi-report-diagnostic/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);inputs=json.load(open('artifacts/safe-gmi-fraction-oracle/input.json'));rng=random.Random(2026111704);records=[];atomic_json(out/'experiment.json',dict(seed=2026111704,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),cases=200,flags=[0,1],cpu=42,seconds=2,memory_bytes=16*1024**3,threads=1))
for case,v in enumerate(inputs):
 d=out/str(case);d.mkdir();text=v['model'];n=len(v['proof']['upper_bounds']);columns=[line.split()[0]for line in text.splitlines()if ' obj 0'in line];costs=[rng.randint(-7,7)for _ in columns]
 for name,cost in zip(columns,costs):text=text.replace(f' {name} obj 0\n',f' {name} obj {cost}\n')
 model=d/'model.mps';model.write_text(text);names,variables,rows,obj,constant=checker.exact_model(model,True)
 def feasible(x,tolerance=F(0)):return all((row['lower']is None or sum(a*x[k]for k,a in row['coefficients'].items())>=row['lower']-tolerance)and(row['upper']is None or sum(a*x[k]for k,a in row['coefficients'].items())<=row['upper']+tolerance)for row in rows)
 def exact_points():
  ints=[k for k in names if variables[k]['integer']];cont=[k for k in names if not variables[k]['integer']];assert 1<=len(cont)<=2
  for assignment in itertools.product(*(range(int(variables[k]['lower']),int(variables[k]['upper'])+1)for k in ints)):
   fixed=dict(zip(ints,assignment));constraints=[]
   for row in rows:
    coefficients=[row['coefficients'].get(k,F(0))for k in cont];shift=sum(row['coefficients'].get(k,F(0))*v for k,v in fixed.items())
    if row['upper']is not None:constraints.append((coefficients,row['upper']-shift))
    if row['lower']is not None:constraints.append(([-a for a in coefficients],shift-row['lower']))
   for j,k in enumerate(cont):
    coefficients=[F(0)]*len(cont);coefficients[j]=1;constraints.append((coefficients,variables[k]['upper']));coefficients=[F(0)]*len(cont);coefficients[j]=-1;constraints.append((coefficients,-variables[k]['lower']))
   vertices=[]
   if len(cont)==1:vertices=[[b/a[0]]for a,b in constraints if a[0]]
   else:
    for (a,b),(c,d)in itertools.combinations(constraints,2):
     determinant=a[0]*c[1]-a[1]*c[0]
     if determinant:vertices.append([(b*c[1]-a[1]*d)/determinant,(a[0]*d-b*c[0])/determinant])
   for vertex in vertices:
    point=dict(fixed);point.update(zip(cont,vertex))
    if feasible(point) and all(sum(a*x for a,x in zip(row,vertex))<=b for row,b in constraints):yield point
 optimum=min(constant+sum(a*x[k]for k,a in obj.items())for x in exact_points())
 for label in (['before','after']if case%2==0 else['after','before']):
  dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_GOMORY_REPORT_ONLY='1',MIP_GOMORY_CUTS='0'if label=='before'else'1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
  for key in ['BINARY_GRAPH','PARITY_EQUALITIES','COVER_CUTS','MIR_CUTS','AGGREGATE_CUTS','CLIQUE_CUTS','THREE_ROW_CUTS','ROUNDING_CUTS','PRIMAL_REPAIR','LP_DIVING','PRIMAL_PROBE','RINS','FEASIBILITY_PUMP','STRONG_BRANCHING','OBJECTIVE_ROUNDING','OBJECTIVE_DOMAIN_REPORT','ROW_RAY_REPORT','ROW_PACKING_REPORT','ROW_SUBGRADIENT_REPORT','DUAL_GUIDED_PRIMAL']:env['MIP_'+key]='0'
  cmd=[str(binary),'solve',str(model),'2'];atomic_json(dest/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_','MKL_'))},input_sha256=sha256(model)));start=time.monotonic()
  with(dest/'result.json').open('w')as stream,(dest/'stderr.log').open('w')as err:
   p=subprocess.Popen(cmd,env=env,stdout=stream,stderr=err,start_new_session=True)
   try:p.wait(timeout=62)
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  atomic_json(dest/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start));assert p.returncode==0;r=json.load(open(dest/'result.json'));proof=checker.validate_cut_proofs(model,r);assert proof['valid']is True,(case,label,proof);assert r['status']not in ['infeasible','unbounded'];point=None
  if r.get('values')is not None:
   point={k:F(str(r['values'][k]))for k in names};assert all((not variables[k]['integer']or abs(x-round(x))<=F(1,1000000))and variables[k]['lower']-F(1,1000000)<=x<=variables[k]['upper']+F(1,1000000)for k,x in point.items());assert feasible(point,F(1,1000000));value=constant+sum(a*point[k]for k,a in obj.items());assert abs(value-F(str(r['objective'])))<=F(1,1000000)
  if r['status']=='optimal':assert point is not None and abs(value-optimum)<=F(1,1000000),(case,label,r['objective'],str(optimum))
  if r.get('dual_bound')is not None:assert F(str(r['dual_bound']))<=optimum+F(1,1000000)
  record=dict(case=case,label=label,status=r['status'],objective=r['objective'],optimum=str(optimum),point=point is not None,proof_valid=True,gomory_added=r['cut_stats']['gomory_added'],bound_proof=r.get('gomory_bound_proof')is not None,nodes=r['nodes'],valid=True);records.append(record);atomic_json(dest/'validation.json',record);atomic_json(out/'progress.json',dict(outputs=len(records),valid=len(records),points=sum(x['point']for x in records),gomory_runs=sum(x['gomory_added']>0 for x in records)))
summary=json.load(open(out/'progress.json'));summary['optimal']=sum(r['status']=='optimal'for r in records);atomic_json(out/'records.json',records);atomic_json(out/'summary.json',summary);atomic_json(out/'exit.json',dict(status='complete'));print(summary)
