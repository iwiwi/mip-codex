from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
solver=sys.argv[1];cpu=int(sys.argv[2]);root=Path.cwd();os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_MEMORY_MIB='16384',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute',MIP_GLPK_DRIVER='/home/ubuntu/mip-codex/target/glpk-benchmark');sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json;from validate_solution import validate
out=root/'artifacts/graph-vertex-connected-native'/solver;out.mkdir(parents=True);records=[]
for case in range(12):
 model=root/'artifacts/graph-vertex-small-share-connected'/f'{case:02d}.mps'
 while not model.exists():time.sleep(1)
 # The producer writes job metadata only after the complete model is closed.
 ready=root/'artifacts/graph-vertex-small-share-connected'/f'{case:02d}-{0 if case%2==0 else 1}'/'job.json'
 while not ready.exists():time.sleep(1)
 d=out/f'{case:02d}';d.mkdir();cmd=[str(root/'.venv/bin/python'),'scripts/comparison_worker.py',solver,str(model),'20',str(d)];atomic_json(d/'job.json',dict(command=cmd,cpu=cpu,seconds=20,watchdog=80,memory_bytes=16*1024**3,threads=1,input_sha256=sha256(model),synthetic=True,optimizer_point_transfer=False));start=time.monotonic();watchdog=False
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True)
  try:p.wait(timeout=80)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog,process_group_killed=watchdog))
 record=dict(case=case,solver=solver,status='watchdog'if watchdog else'execution_error',gmp_valid=None,exit_code=p.returncode)
 if not watchdog and p.returncode==0:
  r=json.load(open(d/'result.json'));v=None
  if r.get('values')is not None:v=validate(model,r);atomic_json(d/'validation.json',v);assert v['valid']is True
  record.update({k:r.get(k)for k in ['status','objective','dual_bound','seconds','nodes','version']});record['gmp_valid']=None if v is None else v['valid']
 records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=12))
