import os,subprocess,json
from pathlib import Path
root=Path('/home/ubuntu/mip-codex'); work=Path('/home/ubuntu/mip-dual-restoration')
records=[]
for profile,count in [('default',500),('larger',300),('binary',500),('mixed_cuts',500),('rounding',500),('projected_rounding',400)]:
 name='phase-one-keep-pricing-synthetic-'+profile
 env=os.environ.copy();env.update(MIP_PHASE1_KEEP_PRICING='1',MIP_PARITY_EQUALITIES='1',MIP_SYNTHETIC_PROFILE=profile,MIP_SYNTHETIC_DIR=str(root/'artifacts'/name),MIP_ABSOLUTE_CHECKER=str(root/'third_party/miplib-checker/bin/solchecker-absolute'))
 with (root/'artifacts'/(name+'.log')).open('w') as out:
  p=subprocess.run(['taskset','-c','2',str(root/'.venv/bin/python'),'scripts/check_synthetic.py',str(count)],cwd=work,env=env,stdout=out,stderr=subprocess.STDOUT)
 records.append(dict(profile=profile,cases=count,exit_code=p.returncode));(root/'artifacts/phase-one-keep-pricing-synthetic-exit.json').write_text(json.dumps(records,indent=2))
 if p.returncode:raise SystemExit(p.returncode)
