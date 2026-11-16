from pathlib import Path
from fractions import Fraction as F
import json,random,subprocess,shutil,sys,os
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
from verify_cover_cuts import exact_model
root=Path.cwd();repo=Path('/home/ubuntu/mip-priced-repair');binary=repo/'target/release/examples/component_objective_oracle';info=require_solver_origin(repo,binary);o=root/'artifacts/component-objective-repair-oracle';o.mkdir();(o/'bin').mkdir();shutil.copy2(binary,o/'bin/helper');shutil.copy2(repo/'examples/component_objective_oracle.rs',o/'source.rs');atomic_json(o/'experiment.json',dict(build_info=info,binary_sha256=sha256(binary),helper_source_sha256=sha256(o/'source.rs'),library_source_reference='artifacts/component-objective-repair-v1-diagnostic',cpu=39,cases=200));rng=random.Random(9107);records=[];os.sched_setaffinity(0,{39})
for case in range(200):
 groups=2+case%2;width=2+(case//2)%2;n=groups*width+1;costs=[1]*n;offset=1 if case%4==2 else 0
 if case%4==1:costs[0]=2
 rows=[]
 for g in range(groups):
  for k in range(2):
   row=[g*width+j for j in range(width)if rng.randrange(2)]
   if not row:row=[g*width]
   rows.append(row)
 d=o/str(case);d.mkdir();p=d/'model.mps';text='NAME oracle\nROWS\n N obj\n'+''.join(f' G r{i}\n'for i in range(len(rows)))+''.join(f' L cap{g}\n'for g in range(groups))+'COLUMNS\n'
 for j in range(n):text+=f' x{j} obj {costs[j]}\n'+''.join(f' x{j} r{i} 1\n'for i,row in enumerate(rows)if j in row)+(f' x{j} cap{j//width} 1\n'if j<n-1 else'')
 text+='RHS\n'+''.join(f' rhs r{i} 1\n'for i in range(len(rows)))+''.join(f' rhs cap{g} {width}\n'for g in range(groups))+f' rhs obj {-offset}\nBOUNDS\n'+''.join(f' BV b x{j}\n'for j in range(n))+'ENDATA\n';p.write_text(text)
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:run=subprocess.run([str(o/'bin/helper'),str(p)],stdout=out,stderr=err,timeout=10)
 assert run.returncode==0;r=json.loads((d/'result.json').read_text());names,variables,constraints,objective,constant=exact_model(p,True);point={name:F(str(r['values'][name]))for name in names};assert all(variables[k]['lower']<=v<=variables[k]['upper']and v.denominator==1 for k,v in point.items())
 def feasible(x):
  for row in constraints:
   value=sum(a*x[k]for k,a in row['coefficients'].items())
   if row['lower']is not None and value<row['lower']or row['upper']is not None and value>row['upper']:return False
  return True
 assert feasible(point);value=constant+sum(a*point[k]for k,a in objective.items());assert value==F(str(r['objective']));opt=min(constant+sum(a*((mask>>names.index(k))&1)for k,a in objective.items())for mask in range(1<<n)if feasible({k:F((mask>>j)&1)for j,k in enumerate(names)}));assert opt<=value<=constant+sum(objective.values());record=dict(case=case,input_sha256=sha256(p),result_sha256=sha256(d/'result.json'),objective=str(value),exact_optimum=str(opt),seed_objective=str(constant+sum(objective.values())),eligible=r.get('component_objective_repair')is not None,valid=True);records.append(record);atomic_json(d/'validation.json',record);atomic_json(o/'summary.json',dict(cases=len(records),valid=len(records),eligible=sum(x['eligible']for x in records),improved=sum(F(x['objective'])<F(x['seed_objective'])for x in records)))
atomic_json(o/'records.json',records);atomic_json(o/'exit.json',dict(status='complete'));print(json.loads((o/'summary.json').read_text()))
