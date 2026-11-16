"""Own LP pricing diagnostics: identical original model, no cuts/heuristics."""
import json,os,resource,subprocess,sys,time
from decimal import Decimal
from fractions import Fraction as F
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256,require_solver_origin
from verify_cover_cuts import exact_model
root=Path.cwd();variant=sys.argv[1];cpu=int(sys.argv[2]);seconds=int(sys.argv[3]);names=sys.argv[4:];os.sched_setaffinity(0,set(range(33,42)));snap=root/'artifacts/runs/clique-cuts-final-20261116';config=json.loads((snap/'config.json').read_text());verify_snapshot(snap,config);before=snap/'snapshot/bin/mip-codex';after=root/'target/release/mip-codex';after_info=require_solver_origin(root,after);manifest={r['name'].removesuffix('.mps.gz'):r for r in config['instances']};records=[]
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1')
for flag in ['MIP_CLIQUE_CUTS','MIP_AGGREGATE_CUTS','MIP_BINARY_GRAPH','MIP_COVER_CUTS','MIP_ROUNDING_CUTS','MIP_LP_DIVING','MIP_PRIMAL_REPAIR','MIP_PRIMAL_PROBE','MIP_RINS','MIP_FEASIBILITY_PUMP','MIP_STRONG_BRANCHING','MIP_OBJECTIVE_ROUNDING','MIP_PROPAGATION','MIP_BOX_PRIMAL']:env[flag]='0'
def restrict():os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in names:
 instance=manifest[name];assert instance['split']=='development';model=Path(instance['absolute_path']);assert sha256(model)==instance['sha256']
 for side,binary in [('before',before),('after',after)]:
  d=root/f'artifacts/static-pricing-{variant}-roots{seconds}'/side/name;d.mkdir(parents=True,exist_ok=True);assert not(d/'experiment.json').exists();dump=d/'root-lp.json';command=[str(binary),'solve',str(model),str(seconds),'1'];atomic_json(d/'experiment.json',dict(variant=variant,side=side,split='development',cpu=cpu,threads=1,memory_bytes=16*1024**3,command=command,binary_sha256=sha256(binary),build_info=config['solver_build_info']if side=='before'else after_info,input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')},purpose='root LP diagnostic only; no native values used'))
  begin=time.monotonic()
  with(d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:r=subprocess.run(command,stdout=stream,stderr=err,env=dict(env,MIP_ROOT_LP_DUMP=str(dump),MIP_LP_TRACE='1'),preexec_fn=restrict,timeout=seconds+60)
  execution=dict(exit_code=r.returncode,wall_seconds=time.monotonic()-begin);atomic_json(d/'exit.json',execution);assert r.returncode==0;r=json.loads((d/'result.json').read_text());record=dict(name=name,side=side,**{k:r.get(k)for k in ['status','dual_bound','nodes','seconds','lp_pivots','lp_refactors','lp_refinement_attempts','lp_refinement_successes','lp_restoration_attempts','lp_restoration_successes']},point_dump=dump.is_file())
  if dump.is_file():
   data=json.loads(dump.read_text(),parse_float=Decimal);ns,vs,rows,cost,offset=exact_model(model,True);assert ns==data['names'];point={n:F(v)for n,v in zip(ns,data['point'])};bound=F(0);violation=F(0)
   for n,v in vs.items():
    if v['lower']is not None:bound=max(bound,v['lower']-point[n])
    if v['upper']is not None:bound=max(bound,point[n]-v['upper'])
   for row in rows:
    a=sum((c*point[n]for n,c in row['coefficients'].items()),F(0))
    if row['lower']is not None:violation=max(violation,row['lower']-a)
    if row['upper']is not None:violation=max(violation,a-row['upper'])
   objective=offset+sum((c*point[n]for n,c in cost.items()),F(0));error=abs(objective-F(data['objective']));valid=bound<=F(1,10**6)and violation<=F(1,10**6)and error<=F(1,10**8)*(1+abs(objective));assert valid;record.update(lp_valid=valid,bound_violation=str(bound),row_violation=str(violation),exact_objective=str(objective),objective_error=str(error))
  atomic_json(d/'summary.json',record);records.append(record);print(json.dumps(record),flush=True)
atomic_json(root/f'artifacts/static-pricing-{variant}-roots{seconds}-cpu{cpu}.json',records)
