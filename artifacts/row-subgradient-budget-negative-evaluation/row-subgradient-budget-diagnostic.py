import os,json,subprocess,signal,resource,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json
root=Path.cwd();o=root/'artifacts/row-subgradient-budget-diagnostic';os.sched_setaffinity(0,{43});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));m={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};records=[]
for name,seedpath in [('seymour1.mps.gz','artifacts/uniform-row-subgradient-joint-long-paired-44/seymour1.mps.gz/after/result.json'),('glass-sc.mps.gz','artifacts/uniform-row-subgradient-joint-long-paired-44/glass-sc.mps.gz/after/result.json'),('sp97ar.mps.gz','artifacts/dual-guided-primal-default-on-long-paired-44/sp97ar.mps.gz/after/result.json'),('sp98ar.mps.gz','artifacts/dual-guided-primal-default-on-sp98-long-paired-43/sp98ar.mps.gz/after/result.json')]:
 r=m[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];seed=root/seedpath;assert seed.is_file()
 for seconds in [.1,.5,2,10]:
  d=o/name/str(seconds);d.mkdir(parents=True);cmd=[str(o/'bin/row_subgradient_budget'),str(model),str(seed),str(seconds)];atomic_json(d/'job.json',dict(command=cmd,input_sha256=r['sha256'],seed_sha256=sha256(seed),cpu=43,threads=1,memory_bytes=17179869184))
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:
   p=subprocess.Popen(cmd,stdout=out,stderr=err,start_new_session=True)
   try:p.wait(timeout=seconds+60)
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
  atomic_json(d/'execution.json',dict(exit_code=p.returncode));assert p.returncode==0
  v=json.loads((d/'stdout.json').read_text());atomic_json(d/'result.json',v.pop('result'));v.update(name=name,directory=str(d));records.append(v);atomic_json(o/'records.json',records);print(name,seconds,v,flush=True)
atomic_json(o/'exit.json',dict(status='complete',records=len(records)))
