from pathlib import Path
from fractions import Fraction as F
import json,subprocess,shutil,sys,os
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
from verify_cover_cuts import exact_model
root=Path.cwd();repo=Path('/home/ubuntu/mip-priced-repair');binary=repo/'target/release/examples/repair_screen_oracle';info=require_solver_origin(repo,binary);o=root/'artifacts/repair-check-screen-stress-oracle';o.mkdir();(o/'bin').mkdir();shutil.copy2(binary,o/'bin/helper');shutil.copy2(repo/'examples/repair_screen_oracle.rs',o/'source.rs');atomic_json(o/'experiment.json',dict(build_info=info,binary_sha256=sha256(binary),helper_source_sha256=sha256(o/'source.rs'),library_source_reference='artifacts/repair-check-screen-v1-diagnostic',cpu=39,cases=30));os.sched_setaffinity(0,{39});records=[]
for case in range(30):
 d=o/str(case);d.mkdir();p=d/'model.mps';n=[400,2000,10000][case%3];rows=[[(i+k)%n for k in range(3)]for i in range(n)];text='NAME stress\nROWS\n N obj\n'+''.join(f' G r{i}\n'for i in range(n))+'COLUMNS\n'
 for j in range(n):text+=f' x{j} obj 1\n'+''.join(f' x{j} r{(j-k)%n} 1\n'for k in range(3))
 text+='RHS\n'+''.join(f' rhs r{i} 1\n'for i in range(n))+'BOUNDS\n'+''.join(f' BV b x{j}\n'for j in range(n))+'ENDATA\n';p.write_text(text);names,variables,constraints,objective,constant=exact_model(p,True);pair=[]
 for label in (['before','after']if case%2==0 else['after','before']):
  dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_REPAIR_CHECK_SCREEN']='0'if label=='before'else'1'
  with(dest/'result.json').open('w')as out,(dest/'stderr.log').open('w')as err:run=subprocess.run([str(o/'bin/helper'),str(p)],env=env,stdout=out,stderr=err,timeout=10)
  assert run.returncode==0;r=json.loads((dest/'result.json').read_text());v=r['result'];valid=True
  if v['values']is not None:
   point={name:F(str(v['values'][name]))for name in names};assert all((variables[k]['lower']is None or variables[k]['lower']<=x)and(variables[k]['upper']is None or x<=variables[k]['upper'])and(not variables[k]['integer']or x.denominator==1)for k,x in point.items())
   for row in constraints:
    x=sum(a*point[k]for k,a in row['coefficients'].items());assert(row['lower']is None or x>=row['lower'])and(row['upper']is None or x<=row['upper'])
   value=constant+sum(a*point[k]for k,a in objective.items());assert value==F(str(v['objective']))
  record=dict(case=case,label=label,input_sha256=sha256(p),result_sha256=sha256(dest/'result.json'),objective=v['objective'],found=v['values']is not None,work=r['work'],moves=r['moves'],seconds=v['seconds'],valid=valid);records.append(record);pair.append(record);atomic_json(dest/'validation.json',record)
 assert pair[0]['work']==pair[1]['work']and pair[0]['moves']==pair[1]['moves']and pair[0]['objective']==pair[1]['objective'],pair
 atomic_json(o/'progress.json',dict(cases=case+1,points=sum(r['found']for r in records),valid=len(records)))
summary=dict(cases=30,outputs=len(records),valid=len(records),points=sum(r['found']for r in records),all_work_moves_objective_equal=True,seconds={label:sum(r['seconds']for r in records if r['label']==label)for label in ['before','after']});atomic_json(o/'records.json',records);atomic_json(o/'summary.json',summary);atomic_json(o/'exit.json',dict(status='complete'));print(summary)
