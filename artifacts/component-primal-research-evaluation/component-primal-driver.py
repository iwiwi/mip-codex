import os,json,resource,time,subprocess,signal,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();os.sched_setaffinity(0,{0});record=next(v for v in json.load(open('data/manifest.json'))['instances']if v['name']=='neos-5052403-cygnet.mps.gz');assert record['split']=='development';model=root/record['path'];assert sha256(model)==record['sha256'];binary=root/'artifacts/component-primal-v1-diagnostic/bin/component-primal';out=root/'artifacts/component-primal-cygnet-120';out.mkdir();cmd=['/usr/bin/time','-v','-o',str(out/'resource.log'),str(binary),str(model),'120',str(out/'components')];env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');atomic_json(out/'experiment.json',dict(command=cmd,cpu=0,seconds=120,watchdog_seconds=180,memory_bytes=16*1024**3,threads=1,input_sha256=record['sha256'],binary_sha256=sha256(binary),environment={k:env[k]for k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']}))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
start=time.monotonic()
with (out/'stdout.log').open('w')as f,(out/'stderr.log').open('w')as e:
 p=subprocess.Popen(cmd,env=env,stdout=f,stderr=e,preexec_fn=restrict,start_new_session=True)
 try:p.wait(timeout=180);watchdog=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
 except BaseException:
  if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
  raise
atomic_json(out/'exit.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog));print(json.load(open(out/'exit.json')))
