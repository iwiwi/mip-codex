from pathlib import Path
import sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();o=r/'artifacts/priced-objective-perturb-diagnostic';b=o/'bin/priced_objective_perturb';assert sha256(b)==json.load(open(o/'experiment.json'))['binary_sha256'];records=[]
for name in ['sp97ar.mps.gz','sp98ar.mps.gz']:
 s=r/'artifacts/priced-objective-continue-v1-long-paired-44'/name/'after/result.json';m=r/'data/raw/benchmark'/name
 for kept,added in [(0.95,0.001),(0.8,0.001),(0.95,0.005),(0.8,0.005)]:
  d=o/name/f'{kept}-{added}';d.mkdir(parents=True);cmd=['taskset','-c','42','timeout','--signal=KILL','65',str(b),str(m),str(s),'2',str(d/'solution.json'),str(kept),str(added)];(d/'input.json').write_text(json.dumps(dict(command=cmd,model_sha256=sha256(m),seed_sha256=sha256(s)),indent=2)+'\n')
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err)
  records.append(dict(name=name,kept=kept,added=added,exit_code=p.returncode,before=json.load(open(s))['objective'],after=json.load(open(d/'stdout.json'))if p.returncode==0 else None));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
