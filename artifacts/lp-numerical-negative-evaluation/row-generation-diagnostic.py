import json,os,sys,subprocess,resource,time,signal
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();work=Path('/home/ubuntu/mip-dual-restoration');binary=root/'target-dual-restoration/release/examples/screen_rows';source=work/'examples/screen_rows.rs';os.sched_setaffinity(0,{40});manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};out=root/'artifacts/row-generation-diagnostic';out.mkdir(exist_ok=False);records=[]
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in ['buildingenergy.mps.gz','ns1760995.mps.gz','square41.mps.gz','supportcase12.mps.gz']:
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];d=out/name;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),str(model),'60','256'];atomic_json(d/'experiment.json',dict(name=name,split='development',input_sha256=r['sha256'],binary_sha256=sha256(binary),example_source_sha256=sha256(source),cpu=40,memory_bytes=16*1024**3,command=cmd));start=time.monotonic()
 with (d/'result.json').open('w')as output,(d/'stderr.log').open('w')as error:
  p=subprocess.Popen(cmd,env=env,stdout=output,stderr=error,start_new_session=True,preexec_fn=restrict)
  try:p.wait(timeout=120);watchdog=False
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start));record=dict(name=name,exit_code=p.returncode,watchdog=watchdog)
 if p.returncode==0:
  value=json.load(open(d/'result.json'));record.update({k:value[k]for k in ['status','objective','dual_bound','rows','active_rows','wall_seconds']});record['lp_runs']=len(value['records'])
 records.append(record);print(json.dumps(record),flush=True);atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',records=len(records)))
