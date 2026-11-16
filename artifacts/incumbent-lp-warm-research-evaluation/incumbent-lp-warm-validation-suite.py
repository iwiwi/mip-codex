import os,subprocess,json
from pathlib import Path
root=Path('/home/ubuntu/mip-codex');work=Path('/home/ubuntu/mip-dual-restoration');records=[]
env=dict(os.environ,MIP_INCUMBENT_LP_WARM='1',MIP_ABSOLUTE_CHECKER=str(root/'third_party/miplib-checker/bin/solchecker-absolute'))
for profile,engines in [('ordinary','dense,revised'),('scaled','dense,revised'),('units','revised')]:
 e=dict(env,MIP_ROOT_PROFILE=profile,MIP_ROOT_ENGINES=engines,MIP_ROOT_DIR=str(root/'artifacts'/('incumbent-lp-warm-v3-root-'+profile)))
 with (root/'artifacts'/('incumbent-lp-warm-v3-root-'+profile+'.log')).open('w') as log:
  p=subprocess.run(['taskset','-c','42',str(root/'.venv/bin/python'),'scripts/check_root_lp.py','100'],cwd=work,env=e,stdout=log,stderr=subprocess.STDOUT)
 records.append(dict(profile=profile,exit_code=p.returncode));(root/'artifacts/incumbent-lp-warm-v3-validation-exit.json').write_text(json.dumps(records,indent=2))
 if p.returncode:raise SystemExit(p.returncode)
for script,key in [('check_serialized_solution.py','MIP_SERIALIZATION_DIR'),('check_transformation_sums.py','MIP_TRANSFORMATION_DIR')]:
 e=dict(env);e[key]=str(root/'artifacts'/('incumbent-lp-warm-v3-'+script[:-3]))
 with (root/'artifacts'/('incumbent-lp-warm-v3-'+script+'.log')).open('w') as log:
  p=subprocess.run(['taskset','-c','42',str(root/'.venv/bin/python'),'scripts/'+script],cwd=work,env=e,stdout=log,stderr=subprocess.STDOUT)
 records.append(dict(script=script,exit_code=p.returncode));(root/'artifacts/incumbent-lp-warm-v3-validation-exit.json').write_text(json.dumps(records,indent=2))
 if p.returncode:raise SystemExit(p.returncode)
