import os,json,sys,subprocess,resource,time,signal,shutil
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256,require_solver_origin
root=Path.cwd();work=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/component-inventory-20261116';binary=out/'inventory';atomic_json(out/'experiment.json',dict(purpose='development-only connected-component classification after fixed variables; no optimization, no holdout read',cpu=45,memory_bytes=16*1024**3,example_sha256=sha256(out/'inventory.rs'),binary_sha256=sha256(binary)));os.sched_setaffinity(0,{45});records=[]

def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for r in json.load(open(root/'data/manifest.json'))['instances']:
 if r['split']!='development':continue
 model=root/r['path'];assert sha256(model)==r['sha256'];d=out/'cases'/r['name'];d.mkdir(parents=True);start=time.monotonic()
 with(d/'stdout.json').open('w')as f,(d/'stderr.log').open('w')as e:
  p=subprocess.Popen([str(binary),str(model)],stdout=f,stderr=e,preexec_fn=restrict,start_new_session=True)
  try:p.wait(timeout=60);watchdog=False
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 result=dict(name=r['name'],input_sha256=r['sha256'],exit_code=p.returncode,watchdog=watchdog,seconds=time.monotonic()-start)
 if p.returncode==0:result['structure']=json.load(open(d/'stdout.json'))
 atomic_json(d/'record.json',result);records.append(result);atomic_json(out/'progress.json',dict(completed=len(records),success=sum('structure'in r for r in records)))
atomic_json(out/'records.json',records);atomic_json(out/'exit.json',dict(status='complete',cases=len(records),success=sum('structure'in r for r in records)));print(json.load(open(out/'exit.json')))
