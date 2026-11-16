import os,json,subprocess,signal,resource,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json
root=Path.cwd();o=root/'artifacts/unit-objective-budget-point-audit';o.mkdir();model=root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz';os.sched_setaffinity(0,{39});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));records=[]
for seconds in [.5,3,10,30]:
 p=root/'artifacts/unit-objective-budget-point-diagnostic'/str(seconds)/'result.json';d=o/str(seconds);d.mkdir();cmd=[str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(p),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]
 with(d/'audit.log').open('w')as stream:
  run=subprocess.Popen(cmd,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
  try:run.wait(timeout=600)
  except subprocess.TimeoutExpired:os.killpg(run.pid,signal.SIGKILL);run.wait()
 assert run.returncode==0;a={k:json.loads((d/(k+'-validation.json')).read_text())for k in ['strict','official']};assert all(v['valid']is True for v in a.values());records.append(dict(seconds=seconds,validation=a));atomic_json(o/'records.json',records)
atomic_json(o/'summary.json',dict(records=len(records),valid=len(records)));print(dict(records=len(records),valid=len(records)))
