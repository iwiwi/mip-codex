import os,json,subprocess,signal,resource,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/dual-numeric-diagnosis';out.mkdir();os.sched_setaffinity(0,{37});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=root/'artifacts/dual-positive-tie-v1-diagnostic/bin/mip-codex';manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']}
for name in ['neos-3754480-nidda.mps.gz','glass4.mps.gz']:
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];d=out/name;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_TRACE='1',MIP_DUAL_POSITIVE_TIE='0',MIP_MIR_CUTS='1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=[str(binary),'solve',str(model),'30'];atomic_json(d/'job.json',dict(command=cmd,input_sha256=r['sha256'],binary_sha256=sha256(binary),environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_','MKL_'))},cpu=37,memory_bytes=16*1024**3));p=None
 with(d/'result.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:
  p=subprocess.Popen(cmd,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
  try:p.wait(timeout=90)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode));print(name,p.returncode,flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
