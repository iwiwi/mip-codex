from pathlib import Path
from fractions import Fraction as F
from importlib.util import spec_from_file_location,module_from_spec
import json,random,itertools,subprocess,signal,resource,os,sys,time
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/safe-gomory-mip-oracle';out.mkdir();os.sched_setaffinity(0,{42});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=root/'artifacts/safe-gomory-joint-diagnostic/bin/mip-codex';spec=spec_from_file_location('checker',root/'artifacts/safe-gomory-joint-diagnostic/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);inputs=json.load(open('artifacts/safe-gomory-rust-oracle/input.json'))[:200];rng=random.Random(2026111704);records=[];atomic_json(out/'experiment.json',dict(seed=2026111704,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),cases=200,flags=[0,1],cpu=42,seconds=2,memory_bytes=16*1024**3,threads=1))
for case,v in enumerate(inputs):
 d=out/str(case);d.mkdir();text=v['model'];n=len(v['proof']['upper_bounds']);columns=[line.split()[0]for line in text.splitlines()if ' obj 0'in line];costs=[rng.randint(-7,7)for _ in columns]
 for name,cost in zip(columns,costs):text=text.replace(f' {name} obj 0\n',f' {name} obj {cost}\n')
 model=d/'model.mps';model.write_text(text);names,variables,rows,obj,constant=checker.exact_model(model,True)
 def feasible(x):return all((row['lower']is None or sum(a*x[k]for k,a in row['coefficients'].items())>=row['lower'])and(row['upper']is None or sum(a*x[k]for k,a in row['coefficients'].items())<=row['upper'])for row in rows)
 optimum=min(constant+sum(a*x[k]for k,a in obj.items())for values in itertools.product(*(range(int(variables[k]['lower']),int(variables[k]['upper'])+1)for k in names))if feasible(x:=dict(zip(names,values))))
 for label in (['before','after']if case%2==0 else['after','before']):
  dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_GOMORY_CUTS='0'if label=='before'else'1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
  for key in ['BINARY_GRAPH','PARITY_EQUALITIES','COVER_CUTS','MIR_CUTS','AGGREGATE_CUTS','CLIQUE_CUTS','THREE_ROW_CUTS','ROUNDING_CUTS','PRIMAL_REPAIR','LP_DIVING','PRIMAL_PROBE','RINS','FEASIBILITY_PUMP','STRONG_BRANCHING','OBJECTIVE_ROUNDING','OBJECTIVE_DOMAIN_REPORT','ROW_RAY_REPORT','ROW_PACKING_REPORT','ROW_SUBGRADIENT_REPORT','DUAL_GUIDED_PRIMAL']:env['MIP_'+key]='0'
  cmd=[str(binary),'solve',str(model),'2'];atomic_json(dest/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_','MKL_'))},input_sha256=sha256(model)));start=time.monotonic()
  with(dest/'result.json').open('w')as stream,(dest/'stderr.log').open('w')as err:
   p=subprocess.Popen(cmd,env=env,stdout=stream,stderr=err,start_new_session=True)
   try:p.wait(timeout=62)
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  atomic_json(dest/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start));assert p.returncode==0;r=json.load(open(dest/'result.json'));proof=checker.validate_cut_proofs(model,r);assert proof['valid']is True,(case,label,proof);assert r['status']not in ['infeasible','unbounded'];point=None
  if r.get('values')is not None:
   point={k:F(str(r['values'][k]))for k in names};assert all(x.denominator==1 and variables[k]['lower']<=x<=variables[k]['upper']for k,x in point.items());assert feasible(point);value=constant+sum(a*point[k]for k,a in obj.items());assert abs(value-F(str(r['objective'])))<=F(1,1000000)
  if r['status']=='optimal':assert point is not None and value==optimum,(case,label,r['objective'],str(optimum))
  if r.get('dual_bound')is not None:assert F(str(r['dual_bound']))<=optimum+F(1,1000000)
  record=dict(case=case,label=label,status=r['status'],objective=r['objective'],optimum=str(optimum),point=point is not None,proof_valid=True,gomory_added=r['cut_stats']['gomory_added'],nodes=r['nodes'],valid=True);records.append(record);atomic_json(dest/'validation.json',record);atomic_json(out/'progress.json',dict(outputs=len(records),valid=len(records),points=sum(x['point']for x in records),gomory_runs=sum(x['gomory_added']>0 for x in records)))
summary=json.load(open(out/'progress.json'));summary['optimal']=sum(r['status']=='optimal'for r in records);atomic_json(out/'records.json',records);atomic_json(out/'summary.json',summary);atomic_json(out/'exit.json',dict(status='complete'));print(summary)
