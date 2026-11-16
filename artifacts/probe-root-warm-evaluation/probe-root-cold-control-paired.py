import json,os,resource,subprocess,time,sys,signal
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();cpu=int(sys.argv[1]);seconds=int(sys.argv[2]);names=sys.argv[3:];os.sched_setaffinity(0,{cpu})
binaries={'before':root/'artifacts/runs/zero-cost-pricing-final-20261116/snapshot/bin/mip-codex','after':root/'target-probe-warm/release/mip-codex'}
out=root/f'artifacts/probe-root-cold-control-paired-{cpu}';out.mkdir(exist_ok=False);manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
atomic_json(out/'experiment.json',dict(seconds=seconds,cpu=cpu,threads=1,memory_bytes=16*1024**3,purpose='development mixed integer pump experiment',binaries={k:dict(path=str(p),sha256=sha256(p),build_info=json.loads(subprocess.check_output([str(p),'build-info'])))for k,p in binaries.items()}))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for index,name in enumerate(names):
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256']
 for label in (['before','after']if index%2==0 else['after','before']):
  binary=binaries[label];d=out/name/label;d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),str(seconds)];atomic_json(d/'job.json',dict(command=cmd,input_sha256=r['sha256'],environment={k:env[k]for k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']},cpu=cpu,seconds=seconds));start=time.monotonic()
  with (d/'stderr.log').open('w')as error,(d/'stdout.json').open('w')as output:
   p=subprocess.Popen(cmd,env=env,stdout=output,stderr=error,start_new_session=True,preexec_fn=restrict)
   try:p.wait(timeout=seconds+60);watchdog=False
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
   except BaseException:
    if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
    raise
  atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog))
  if p.returncode==0:
   v=json.loads((d/'stdout.json').read_text());atomic_json(d/'result.json',v);print(name,label,v['status'],v['objective'],v['dual_bound'],v.get('probe'),flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
