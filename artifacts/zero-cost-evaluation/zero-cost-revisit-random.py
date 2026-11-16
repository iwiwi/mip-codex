"""Planted constant-objective LPs, independently checked with Fraction."""
import os,json,sys,random,subprocess,resource,signal,time
from pathlib import Path
from decimal import Decimal
from fractions import Fraction as F
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from verify_cover_cuts import exact_model
os.sched_setaffinity(0,{0});root=Path.cwd();out=root/'artifacts/zero-cost-revisit-random';out.mkdir(exist_ok=False);rng=random.Random(202611160410);binaries={'before':root/'artifacts/runs/matrix-rhs-final-20261116/snapshot/bin/mip-codex','after':root/'target-zero-cost/release/mip-codex'};atomic_json(out/'experiment.json',dict(seed=202611160410,cases=100,cpu=0,seconds=5,threads=1,memory_bytes=16*1024**3,binaries={k:dict(path=str(p),sha256=sha256(p),build_info=json.loads(subprocess.check_output([str(p),'build-info'])))for k,p in binaries.items()}));records=[]
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for case in range(100):
 n=rng.randint(5,40);m=rng.randint(8,50);bounds=[(rng.randint(-2,0),rng.randint(1,4))for _ in range(n)];planted=[rng.randint(lo,hi)for lo,hi in bounds];offset=rng.randint(-4,4);integers=[rng.random()<.5 for _ in range(n)];rows=[]
 for i in range(m):
  a=[rng.randint(-7,7)if rng.random()<.35 else 0 for _ in range(n)];kind=rng.choice(['L','G','E']);b=sum(a[j]*planted[j]for j in range(n))+rng.randint(0,5)*(1 if kind=='L'else-1 if kind=='G'else 0);rows.append((a,b,kind))
 lines=['NAME zero','ROWS',' N obj']+[f' {kind} r{i}'for i,(_,_,kind)in enumerate(rows)]+['COLUMNS']
 for j in range(n):lines += [f' x{j} obj 0']+[f' x{j} r{i} {a[j]}'for i,(a,_,_)in enumerate(rows)if a[j]]
 lines+=['RHS',f' rhs obj {-offset}']+[f' rhs r{i} {b}'for i,(_,b,_)in enumerate(rows)]+['BOUNDS']
 for j,(lo,hi)in enumerate(bounds):lines += [f' {"LI"if integers[j]else"LO"} bd x{j} {lo}',f' UP bd x{j} {hi}']
 lines+=['ENDATA'];directory=out/f'{case:03}';directory.mkdir();model=directory/'model.mps';model.write_text('\n'.join(lines)+'\n');names,variables,exact_rows,_,_=exact_model(model,True)
 for label,binary in binaries.items():
  d=directory/label;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
  for flag in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+flag]='0'
  env.update(MIP_ROOT_LP_DUMP=str(d/'root-lp.json'),OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');start=time.monotonic()
  with (d/'result.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:
   proc=subprocess.Popen([str(binary),'solve',str(model),'5','1'],env=env,stdout=stdout,stderr=stderr,start_new_session=True,preexec_fn=restrict)
   try:proc.wait(timeout=15)
   except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
  r=dict(case=case,variant=label,valid=False,exit_code=proc.returncode,wall_seconds=time.monotonic()-start,input_sha256=sha256(model))
  try:
   result=json.loads((d/'result.json').read_text());data=json.loads((d/'root-lp.json').read_text(),parse_float=Decimal);assert data['names']==names;point={k:F(v)for k,v in zip(names,data['point'])};violation=F(0)
   for name,v in variables.items():
    if v['lower']is not None:violation=max(violation,v['lower']-point[name])
    if v['upper']is not None:violation=max(violation,point[name]-v['upper'])
   for row in exact_rows:
    a=sum((c*point[k]for k,c in row['coefficients'].items()),F(0))
    if row['lower']is not None:violation=max(violation,row['lower']-a)
    if row['upper']is not None:violation=max(violation,a-row['upper'])
   assert violation<=F(1,10**6);assert F(data['objective'])==offset;assert result['dual_bound']==offset;r.update(valid=True,violation=str(violation),seconds=result['seconds'],pivots=result['lp_pivots'],pricing_calls=result.get('lp_feasibility_pricing_calls'),pricing_fallbacks=result.get('lp_pricing_fallbacks'))
  except Exception as error:r['error']=f'{type(error).__name__}: {error}'
  records.append(r);atomic_json(out/'records.json',records);print(json.dumps(r),flush=True)
summary=dict(settings=100,outputs=len(records),valid=sum(r['valid']for r in records),invalid=sum(not r['valid']for r in records));atomic_json(out/'summary.json',summary);print(json.dumps(summary));assert not summary['invalid']
