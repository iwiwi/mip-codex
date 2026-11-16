from pathlib import Path
import sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();o=r/'artifacts/priced-dual-subgradient-diagnostic';b=o/'bin/priced_dual_subgradient';assert sha256(b)==json.load(open(o/'experiment.json'))['binary_sha256'];records=[]
for name in ['sp97ar.mps.gz','sp98ar.mps.gz']:
 s=r/'artifacts/priced-objective-continue-v1-long-paired-44'/name/'after/result.json';m=r/'data/raw/benchmark'/name
 for factor in ['0.1','0.5','1','2']:
  d=o/name/factor;d.mkdir(parents=True);cmd=['taskset','-c','37','timeout','--signal=KILL','65',str(b),str(m),str(s),'2',factor];(d/'input.json').write_text(json.dumps(dict(command=cmd,seed_sha256=sha256(s),model_sha256=sha256(m)),indent=2)+'\n')
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err)
  a=json.load(open(d/'stdout.json'))if p.returncode==0 else None;records.append(dict(name=name,factor=factor,exit_code=p.returncode,result={k:v for k,v in a.items()if k!='multipliers'}if a else None));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
