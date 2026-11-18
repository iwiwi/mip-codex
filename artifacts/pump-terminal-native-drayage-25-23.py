from pathlib import Path
import os,sys,json,time,subprocess,resource,signal
root=Path.cwd();os.sched_setaffinity(0,{43});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_MEMORY_MIB='16384',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json
from validate_solution import validate
model=root/'data/raw/benchmark/drayage-25-23.mps.gz';assert any(r['path']=='data/raw/benchmark/drayage-25-23.mps.gz'and r['split']=='development'for r in json.load(open('data/manifest.json'))['instances']);out=root/'artifacts/pump-terminal-native-drayage-25-23';out.mkdir();atomic_json(out/'experiment.json',dict(seconds=10,watchdog=70,cpu=43,threads=1,memory_bytes=16*1024**3,input_sha256=sha256(model),selection='development example with new terminal pump primal; reporting comparison only; no point transfer'));records=[]
for solver in ['highs','scip','glpk']:
 d=out/solver;d.mkdir();cmd=[str(root/'.venv/bin/python'),str(root/'scripts/comparison_worker.py'),solver,str(model),'10',str(d)];atomic_json(d/'job.json',dict(command=cmd));start=time.monotonic();watchdog=False
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True);atomic_json(d/'process.json',dict(pid=p.pid,process_group=p.pid,started_unix=time.time()))
  try:p.wait(timeout=70)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog,process_group_killed=watchdog))
 if watchdog or p.returncode!=0:record=dict(solver=solver,status='watchdog'if watchdog else'execution_error',exit_code=p.returncode,primal_accepted=False)
 else:
  r=json.load(open(d/'result.json'));v=None
  if r.get('values')is not None:
   try:v=validate(model,r)
   except Exception as error:v=dict(valid=None,error=str(error))
   atomic_json(d/'validation.json',v)
  record=dict(solver=solver,**{k:r.get(k)for k in ['status','objective','dual_bound','seconds','nodes','version']},gmp_valid=None if v is None else v['valid'],primal_accepted=v is not None and v['valid']is True)
 records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',cases=len(records),accepted_primal_points=sum(r['primal_accepted']for r in records)))
