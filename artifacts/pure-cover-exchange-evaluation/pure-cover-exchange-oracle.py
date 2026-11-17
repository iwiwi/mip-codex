from pathlib import Path
from fractions import Fraction as F
import json,subprocess,shutil,sys,os
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
from verify_cover_cuts import exact_model
root=Path.cwd();repo=Path('/home/ubuntu/mip-objective-repair');binary=repo/'target/release/examples/cover_exchange_oracle';info=require_solver_origin(repo,binary);o=root/'artifacts/pure-cover-exchange-oracle';o.mkdir();(o/'bin').mkdir();shutil.copy2(binary,o/'bin/helper');shutil.copy2(repo/'examples/cover_exchange_oracle.rs',o/'helper.rs');atomic_json(o/'experiment.json',dict(build_info=info,binary_sha256=sha256(binary),helper_source_sha256=sha256(o/'helper.rs'),source_reference='artifacts/pure-cover-exchange-v1-diagnostic',cpu=39,cases=300,outputs=600));records=[];os.sched_setaffinity(0,{39})
for case in range(300):
 d=o/str(case);d.mkdir();p=d/'model.mps';source=root/f'artifacts/pure-cover-greedy-oracle/{case}';v=json.load(open(source/'validation.json'));assert sha256(source/'model.mps')==v['input_sha256'];shutil.copy2(source/'model.mps',p);names,variables,constraints,objective,constant=exact_model(p,True)
 def feasible(point):
  return all((row['lower']is None or sum(a*point[k]for k,a in row['coefficients'].items())>=row['lower'])and(row['upper']is None or sum(a*point[k]for k,a in row['coefficients'].items())<=row['upper'])for row in constraints)
 optimum=min(constant+sum(a*((bits>>names.index(k))&1)for k,a in objective.items())for bits in range(1<<len(names))if feasible({k:F((bits>>j)&1)for j,k in enumerate(names)}))if case<200 else None;pair={}
 for label in (['before','after']if case%2==0 else['after','before']):
  dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_PURE_COVER_EXCHANGE']='0'if label=='before'else'1'
  with(dest/'result.json').open('w')as out,(dest/'stderr.log').open('w')as err:proc=subprocess.run([str(o/'bin/helper'),str(p)],stdout=out,stderr=err,env=env,timeout=10)
  assert proc.returncode==0;r=json.load(open(dest/'result.json'))
  if case<200:
   assert r['values']is not None;point={k:F(str(r['values'][k]))for k in names};assert all(x.denominator==1 and 0<=x<=1 for x in point.values())and feasible(point);value=constant+sum(a*point[k]for k,a in objective.items());assert value==F(str(r['objective']))and value>=optimum
  else:assert r['values']is None and r['status']=='unknown'
  record=dict(case=case,label=label,input_sha256=sha256(p),result_sha256=sha256(dest/'result.json'),supported=case<200,point=r['values']is not None,objective=r['objective'],enumerated_minimum=str(optimum),calls=r['pure_cover_greedy']['calls'],valid=True);records.append(record);pair[label]=record;atomic_json(dest/'validation.json',record)
 if case<200:assert pair['after']['objective']<=pair['before']['objective']
 atomic_json(o/'progress.json',dict(cases=case+1,outputs=len(records),valid=len(records),points=sum(r['point']for r in records)))
atomic_json(o/'records.json',records);summary=dict(cases=300,outputs=len(records),valid=len(records),points=sum(r['point']for r in records),unsupported_outputs=sum(not r['supported']for r in records),improved=sum(next(r['objective']for r in records if r['case']==c and r['label']=='after')<next(r['objective']for r in records if r['case']==c and r['label']=='before')for c in range(200)));atomic_json(o/'summary.json',summary);atomic_json(o/'exit.json',dict(status='complete'));print(summary)
