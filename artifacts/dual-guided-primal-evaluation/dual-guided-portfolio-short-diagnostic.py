from pathlib import Path
import sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();o=r/'artifacts/dual-guided-portfolio-short-diagnostic';b=r/'artifacts/dual-guided-portfolio-diagnostic/bin/helper';assert sha256(b)==json.load(open(r/'artifacts/dual-guided-portfolio-diagnostic/experiment.json'))['binary_sha256'];o.mkdir();records=[]
for name in ['fast0507.mps.gz','n3div36.mps.gz','sp97ar.mps.gz','sp98ar.mps.gz','eil33-2.mps.gz']:
 p=r/'artifacts/priced-row-subgradient-v1-paired-41'/name/'after/result.json'
 if not p.exists():p=r/'artifacts/priced-row-subgradient-development-screen/cases'/name/'after/result.json'
 for mode in ['zero','portfolio']:
  d=o/name/mode;d.mkdir(parents=True);m=r/'data/raw/benchmark'/name;cmd=['taskset','-c','37','timeout','--signal=KILL','65',str(b),str(m),str(p),'.1',str(d/'solution.json'),mode];(d/'input.json').write_text(json.dumps(dict(command=cmd,model_sha256=sha256(m),seed_sha256=sha256(p)),indent=2)+'\n')
  with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:q=subprocess.run(cmd,stdout=out,stderr=err)
  a=json.load(open(d/'stdout.json'))if q.returncode==0 else None;records.append(dict(name=name,mode=mode,before=json.load(open(p))['objective'],exit_code=q.returncode,result=a));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
