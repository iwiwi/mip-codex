import json,os,resource,subprocess,sys,time,signal
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
cpu=int(sys.argv[1]);seconds=int(sys.argv[2]);names=sys.argv[3:];os.sched_setaffinity(0,{cpu});root=Path.cwd();out=root/f'artifacts/phase-one-delayed-pricing-mip-{cpu}';out.mkdir(exist_ok=False);binary=root/'artifacts/phase-one-delayed-pricing-diagnostic/bin/mip-codex';manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};atomic_json(out/'experiment.json',dict(cpu=cpu,seconds=seconds,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),purpose='development MIP off/immediate/delayed Phase-I pricing'))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for index,name in enumerate(names):
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];modes=[('off','0','0'),('immediate','1','0'),('delayed','1','1')];modes=modes[index%3:]+modes[:index%3]
 for mode,keep,delay in modes:
  d=out/name/mode;d.mkdir(parents=True);e={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};e.update(MIP_PHASE1_KEEP_PRICING=keep,MIP_PHASE1_DELAYED_PRICING=delay,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');atomic_json(d/'job.json',dict(input_sha256=r['sha256'],environment={k:v for k,v in e.items()if k.startswith('MIP_')or k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']}));start=time.monotonic()
  with(d/'result.json').open('w')as f,(d/'stderr.log').open('w')as err:
   p=subprocess.Popen([str(binary),'solve',str(model),str(seconds)],env=e,stdout=f,stderr=err,preexec_fn=restrict,start_new_session=True)
   try:p.wait(timeout=seconds+60)
   except BaseException:
    if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
    raise
  assert p.returncode==0;v=json.load(open(d/'result.json'));atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start));print(name,mode,v['status'],v['objective'],v['dual_bound'],flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
