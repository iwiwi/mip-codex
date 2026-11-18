from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
solver=sys.argv[1];cpu=int(sys.argv[2]);root=Path.cwd();os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_MEMORY_MIB='16384',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute',MIP_GLPK_DRIVER='/home/ubuntu/mip-codex/target/glpk-benchmark');sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json;from validate_solution import validate
out=root/'artifacts/farkas-row-projection-native-long'/solver;out.mkdir(parents=True);manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for name in ['neos-2657525-crna.mps.gz']:
 m=manifest[name];assert m['split']=='development';model=root/m['path'];assert sha256(model)==m['sha256'];d=out/name;d.mkdir();cmd=[str(root/'.venv/bin/python'),'scripts/comparison_worker.py',solver,str(model),'300',str(d)];atomic_json(d/'job.json',dict(command=cmd,cpu=cpu,seconds=300,watchdog=360,memory_bytes=16*1024**3,threads=1,input_sha256=m['sha256'],development_only=True,optimizer_point_transfer=False));start=time.monotonic();watchdog=False
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True)
  try:p.wait(timeout=360)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog,process_group_killed=watchdog));record=dict(name=name,solver=solver,status='watchdog'if watchdog else'execution_error',gmp_valid=None,exit_code=p.returncode)
 if not watchdog and p.returncode==0:
  r=json.load(open(d/'result.json'));v=None
  if r.get('values')is not None:v=validate(model,r);atomic_json(d/'validation.json',v)
  record.update({k:r.get(k)for k in ['status','dual_bound','seconds','nodes','version']});record['native_objective']=r.get('objective');record['objective']=r.get('objective')if v is not None and v['valid']is True else None;record['gmp_valid']=None if v is None else v['valid']
 records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=1))
