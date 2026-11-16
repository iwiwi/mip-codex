from pathlib import Path
import os,subprocess,json,time
root=Path.cwd();work=Path('/home/ubuntu/mip-aggregate-three');records=[]
env=os.environ.copy();env['MIP_ABSOLUTE_CHECKER']=str(root/'third_party/miplib-checker/bin/solchecker-absolute')
cmd=[str(root/'.venv/bin/python'),str(root/'artifacts/aggregate-three-combined-synthetic.py')]
with (root/'artifacts/aggregate-three-combined-synthetic-driver.log').open('w')as out:p=subprocess.run(cmd,cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT)
records.append(dict(stage='2700 synthetic',exit_code=p.returncode));assert p.returncode==0
stages=[('root-'+profile,'scripts/check_root_lp.py',['100'],{'MIP_ROOT_PROFILE':profile,'MIP_ROOT_ENGINES':'revised'if profile=='units'else'dense,revised','MIP_ROOT_DIR':str(root/'artifacts'/('aggregate-three-combined-root-'+profile))})for profile in ['ordinary','scaled','units']]
stages += [('serialization','scripts/check_serialized_solution.py',[],{'MIP_SERIALIZATION_DIR':str(root/'artifacts/aggregate-three-combined-serialization')}),('transformation','scripts/check_transformation_sums.py',[],{'MIP_TRANSFORMATION_DIR':str(root/'artifacts/aggregate-three-combined-transformation')})]
for label,script,args,options in stages:
 with (root/'artifacts'/('aggregate-three-combined-'+label+'.log')).open('w')as out:p=subprocess.run(['taskset','-c','2',str(root/'.venv/bin/python'),script,*args],cwd=work,env=dict(env,**options),stdout=out,stderr=subprocess.STDOUT)
 records.append(dict(stage=label,exit_code=p.returncode));(root/'artifacts/aggregate-three-combined-regression-exit.json').write_text(json.dumps(records,indent=2));assert p.returncode==0,(label,p.returncode)
