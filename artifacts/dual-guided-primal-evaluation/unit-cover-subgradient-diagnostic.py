from pathlib import Path
import sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();f=r/'artifacts/priced-dual-subgradient-diagnostic';o=r/'artifacts/unit-cover-subgradient-diagnostic';o.mkdir();b=f/'bin/priced_dual_subgradient';assert sha256(b)==json.load(open(f/'experiment.json'))['binary_sha256'];records=[]
for name in ['seymour1.mps.gz','glass-sc.mps.gz','neos-5052403-cygnet.mps.gz']:
 p=r/'artifacts/priced-row-subgradient-development-screen/cases'/name/'after/result.json';v=json.load(open(p));assert v['values']is not None
 for seconds in ['0.1','0.5']:
  d=o/name/seconds;d.mkdir(parents=True);m=r/'data/raw/benchmark'/name;cmd=['taskset','-c','37','timeout','--signal=KILL','65',str(b),str(m),str(p),seconds,'2'];(d/'input.json').write_text(json.dumps(dict(command=cmd,model_sha256=sha256(m),seed_sha256=sha256(p),binary_sha256=sha256(b),seed_primal=v['objective'],existing_dual=v['dual_bound']),indent=2)+'\n')
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:q=subprocess.run(cmd,stdout=out,stderr=err)
  a=json.load(open(d/'stdout.json'))if q.returncode==0 else None;records.append(dict(name=name,seconds=seconds,exit_code=q.returncode,existing_dual=v['dual_bound'],result={k:x for k,x in a.items()if k!='multipliers'}if a else None));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
