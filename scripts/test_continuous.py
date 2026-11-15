#!/usr/bin/env python3
"""Continuous LP differential test against scipy/HiGHS, including free variables."""
import json,pathlib,random,subprocess,tempfile
import numpy as np
from scipy.optimize import linprog
rng=random.Random(2026111502);solver=str(pathlib.Path('target/release/mip-codex').resolve());count=400
with tempfile.TemporaryDirectory() as directory:
 for case in range(count):
  n=rng.randrange(1,9);m=rng.randrange(0,13);cost=[rng.randrange(-8,9) for _ in range(n)];a=[[rng.randrange(-5,6) for _ in range(n)] for _ in range(m)];b=[rng.randrange(-10,16) for _ in range(m)];bounds=[rng.choice([(None,None),(0,None),(None,3),(-2,4),(0,1),(1,1)]) for _ in range(n)];maximize=rng.choice([False,True]);sense=-1 if maximize else 1;offset=rng.randrange(-5,6)
  equal=rng.randrange(m+1);oracle=linprog(np.array(cost)*sense,A_ub=np.array(a[equal:]) if equal<m else None,b_ub=b[equal:] if equal<m else None,A_eq=np.array(a[:equal]) if equal else None,b_eq=b[:equal] if equal else None,bounds=bounds,method='highs')
  lines=['NAME random','OBJSENSE',' MAX' if maximize else ' MIN','ROWS',' N obj']+[f' {"E" if i<equal else "L"} r{i}' for i in range(m)]+['COLUMNS']
  for j in range(n):
   lines.append(f' x{j} obj {cost[j]}');lines.extend(f' x{j} r{i} {a[i][j]}' for i in range(m))
  lines+=['RHS',f' rhs obj {-offset}']+[f' rhs r{i} {b[i]}' for i in range(m)]+['BOUNDS']
  for j,(lo,up) in enumerate(bounds):
   if lo is None and up is None:lines.append(f' FR b x{j}')
   elif lo is None:lines.extend([f' MI b x{j}',f' UP b x{j} {up}'])
   else:
    lines.append(f' LO b x{j} {lo}')
    if up is not None:lines.append(f' UP b x{j} {up}')
  lines+=['ENDATA'];path=pathlib.Path(directory)/'test.mps';path.write_text('\n'.join(lines)+'\n')
  s=json.loads(subprocess.run([solver,'solve',str(path),'2'],text=True,capture_output=True,check=True).stdout)
  expected={0:'optimal',2:'infeasible',3:'unbounded_relaxation'}.get(oracle.status)
  if expected is None:raise AssertionError((case,oracle.message))
  if s['status']!=expected or expected=='optimal' and abs(s['objective']-(sense*oracle.fun+offset))>1e-6*max(1,abs(oracle.fun)):
   pathlib.Path('logs/continuous-failure.mps').write_text(path.read_text());raise AssertionError((case,oracle.message,oracle.fun,s))
print(json.dumps(dict(cases=count,seed=2026111502,result='passed')))
