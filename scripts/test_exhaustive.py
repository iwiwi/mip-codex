#!/usr/bin/env python3
"""Fixed-seed independent exhaustive oracle for small bounded integer MILPs."""
import itertools,json,pathlib,random,subprocess,tempfile
rng=random.Random(20261115);solver=str(pathlib.Path('target/debug/mip-codex').resolve());count=300
with tempfile.TemporaryDirectory() as directory:
 for case in range(count):
  n=rng.randrange(1,7);m=rng.randrange(0,8);cost=[rng.randrange(-7,8) for _ in range(n)];upper=[rng.randrange(1,4) for _ in range(n)];a=[[rng.randrange(-4,5) for _ in range(n)] for _ in range(m)];b=[rng.randrange(-8,16) for _ in range(m)]
  best=None
  for x in itertools.product(*(range(u+1) for u in upper)):
   if all(sum(v*y for v,y in zip(row,x))<=rhs for row,rhs in zip(a,b)):
    obj=sum(v*y for v,y in zip(cost,x));best=obj if best is None else min(best,obj)
  lines=['NAME random','ROWS',' N obj']+[f' L r{i}' for i in range(m)]+['COLUMNS']
  for j in range(n):
   lines.append(f' x{j} obj {cost[j]}')
   lines.extend(f' x{j} r{i} {a[i][j]}' for i in range(m))
  lines+=['RHS']+[f' rhs r{i} {b[i]}' for i in range(m)]+['BOUNDS']+[f' UI bound x{j} {upper[j]}' for j in range(n)]+['ENDATA']
  path=pathlib.Path(directory)/'test.mps';path.write_text('\n'.join(lines)+'\n')
  result=subprocess.run([solver,'solve',str(path),'2'],capture_output=True,text=True,check=True);s=json.loads(result.stdout)
  assert s['status']==('infeasible' if best is None else 'optimal'),(case,best,s)
  assert s['objective']==best,(case,best,s)
  if best is not None:
   x=s['values'];assert all(abs(v-round(v))<1e-7 and 0<=v<=u for v,u in zip(x,upper));assert all(sum(v*y for v,y in zip(row,x))<=rhs+1e-7 for row,rhs in zip(a,b))
print(json.dumps(dict(cases=count,seed=20261115,result='passed')))
