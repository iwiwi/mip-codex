from pathlib import Path
import json,subprocess,sys
sys.path.insert(0,'scripts');from benchmark_common import sha256
r=Path.cwd();f=r/'artifacts/priced-objective-cut-diagnostic';o=r/'artifacts/priced-objective-cut-long-diagnostic';o.mkdir();b=f/'bin/priced_objective_cut';assert sha256(b)==json.load(open(f/'experiment.json'))['binary_sha256'];records=[]
for name in ['n3div36.mps.gz','sp97ar.mps.gz','sp98ar.mps.gz','tbfp-network.mps.gz']:
 s=r/'artifacts/priced-portfolio-v1-paired-41'/name/'after/result.json';m=r/'data/raw/benchmark'/name;d=o/name;d.mkdir();cmd=['taskset','-c','42','timeout','--signal=KILL','70',str(b),str(m),str(s),'10',str(d/'solution.json')];(d/'input.json').write_text(json.dumps(dict(command=cmd,seed_sha256=sha256(s),model_sha256=sha256(m),binary_sha256=sha256(b)),indent=2)+'\n')
 with(d/'stdout.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err)
 records.append(dict(name=name,exit_code=p.returncode,before=json.load(open(s))['objective'],after=json.load(open(d/'stdout.json'))if p.returncode==0 else None));(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n')
