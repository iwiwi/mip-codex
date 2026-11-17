from pathlib import Path
from fractions import Fraction as F
import json,random,subprocess,shutil,sys,os
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
from verify_cover_cuts import exact_model
root=Path.cwd();repo=Path('/home/ubuntu/mip-objective-repair');binary=repo/'target/release/examples/pure_cover_oracle';info=require_solver_origin(repo,binary);o=root/'artifacts/pure-cover-greedy-oracle';o.mkdir();(o/'bin').mkdir();shutil.copy2(binary,o/'bin/helper');shutil.copy2(repo/'examples/pure_cover_oracle.rs',o/'helper.rs');atomic_json(o/'experiment.json',dict(build_info=info,binary_sha256=sha256(binary),helper_source_sha256=sha256(o/'helper.rs'),source_reference='artifacts/pure-cover-greedy-v2-diagnostic',cpu=39,cases=300));rng=random.Random(181);records=[];os.sched_setaffinity(0,{39})
for case in range(300):
 n=3+case%7;rows=[];costs=[rng.randrange(5)for _ in range(n)]
 for i in range(2+case%9):
  row=[j for j in range(n)if rng.randrange(2)]
  rows.append(row or[case%n])
 unsupported=case>=200;kind=case%4 if unsupported else None
 if kind==3:costs[0]=-1
 d=o/str(case);d.mkdir();p=d/'model.mps';text='NAME oracle\n'+('OBJSENSE\n MAX\n'if kind==0 else'')+'ROWS\n N obj\n'+''.join(f' G r{i}\n'for i in range(len(rows)))+(' L cap\n'if kind==1 else'')+'COLUMNS\n'
 for j in range(n):text+=f' x{j} obj {costs[j]}\n'+''.join(f' x{j} r{i} 1\n'for i,row in enumerate(rows)if j in row)+(f' x{j} cap 1\n'if kind==1 else'')
 text+='RHS\n'+''.join(f' rhs r{i} 1\n'for i in range(len(rows)))+(f' rhs cap {n}\n'if kind==1 else'')+'BOUNDS\n'+''.join((f' LO b x{j} 0\n UP b x{j} 1\n'if kind==2 else f' BV b x{j}\n')for j in range(n))+'ENDATA\n';p.write_text(text)
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:proc=subprocess.run([str(o/'bin/helper'),str(p)],stdout=out,stderr=err,timeout=10)
 assert proc.returncode==0;r=json.load(open(d/'result.json'));names,variables,constraints,objective,constant=exact_model(p,True)
 def feasible(point):
  return all((row['lower']is None or sum(a*point[k]for k,a in row['coefficients'].items())>=row['lower'])and(row['upper']is None or sum(a*point[k]for k,a in row['coefficients'].items())<=row['upper'])for row in constraints)
 optimum=min(constant+sum(a*((bits>>names.index(k))&1)for k,a in objective.items())for bits in range(1<<n)if feasible({k:F((bits>>j)&1)for j,k in enumerate(names)}))
 if not unsupported:
  assert r['values']is not None;point={k:F(str(r['values'][k]))for k in names};assert all(x.denominator==1 and 0<=x<=1 for x in point.values())and feasible(point);value=constant+sum(a*point[k]for k,a in objective.items());assert value==F(str(r['objective']))and value>=optimum
 else:assert r['values']is None and r['status']=='unknown'
 record=dict(case=case,input_sha256=sha256(p),result_sha256=sha256(d/'result.json'),supported=not unsupported,point=r['values']is not None,objective=r['objective'],enumerated_minimum=str(optimum),valid=True);records.append(record);atomic_json(d/'validation.json',record);atomic_json(o/'progress.json',dict(cases=len(records),valid=len(records),points=sum(x['point']for x in records)))
atomic_json(o/'records.json',records);atomic_json(o/'summary.json',dict(cases=len(records),valid=len(records),points=sum(x['point']for x in records),unsupported=sum(not x['supported']for x in records)));atomic_json(o/'exit.json',dict(status='complete'));print(json.load(open(o/'summary.json')))
