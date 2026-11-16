from pathlib import Path
import sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();f=r/'artifacts/priced-dual-subgradient-diagnostic';o=r/'artifacts/priced-dual-subgradient-short-diagnostic';o.mkdir();b=f/'bin/priced_dual_subgradient';assert sha256(b)==json.load(open(f/'experiment.json'))['binary_sha256'];records=[]
for name in ['sp97ar.mps.gz','sp98ar.mps.gz']:
 s=r/'artifacts/priced-objective-budget-v1-paired-41'/name/'after/result.json';m=r/'data/raw/benchmark'/name
 for seconds in ['0.1','0.5','2']:
  d=o/name/seconds;d.mkdir(parents=True);cmd=['taskset','-c','37','timeout','--signal=KILL','65',str(b),str(m),str(s),seconds,'2'];(d/'input.json').write_text(json.dumps(dict(command=cmd,seed_sha256=sha256(s),model_sha256=sha256(m),binary_sha256=sha256(b)),indent=2)+'\n')
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err)
  a=json.load(open(d/'stdout.json'))if p.returncode==0 else None;records.append(dict(name=name,seconds=seconds,exit_code=p.returncode,result={k:v for k,v in a.items()if k!='multipliers'}if a else None));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
