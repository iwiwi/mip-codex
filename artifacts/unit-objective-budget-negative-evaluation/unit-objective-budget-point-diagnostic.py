import os,json,subprocess,signal,resource,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();o=root/'artifacts/unit-objective-budget-point-diagnostic';o.mkdir();os.sched_setaffinity(0,{43});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));model=root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz';seed=root/'artifacts/uniform-row-subgradient-final-development-screen/cases/neos-5052403-cygnet.mps.gz/after/result.json';binary=root/'artifacts/unit-objective-budget-v1-diagnostic/bin/unit_objective_budget';records=[]
for seconds in [.5,3,10,30]:
 d=o/str(seconds);d.mkdir();cmd=[str(binary),str(model),str(seed),str(seconds),str(d/'result.json')];atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),seed_sha256=sha256(seed),binary_sha256=sha256(binary),cpu=43,threads=1,memory_bytes=17179869184))
 with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:
  p=subprocess.Popen(cmd,stdout=out,stderr=err,start_new_session=True)
  try:p.wait(timeout=seconds+60)
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode));assert p.returncode==0;v=json.loads((d/'stdout.json').read_text());records.append(dict(requested_seconds=seconds,**v));atomic_json(o/'records.json',records);print(records[-1],flush=True)
atomic_json(o/'exit.json',dict(status='complete',records=len(records)))
