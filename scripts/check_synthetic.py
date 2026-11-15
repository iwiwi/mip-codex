#!/usr/bin/env python3
"""固定seedの有界小MILP/LPをHiGHSと比較。純整数では全列挙も照合。"""
import hashlib
import itertools
import json
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import highspy
from validate_solution import validate

root = Path(__file__).resolve().parents[1]
rng = random.Random(20261115)
count = int(sys.argv[1]) if len(sys.argv)>1 else 500
failed = 0
outdir = root/'artifacts/synthetic'
outdir.mkdir(exist_ok=True)
for case in range(count):
    n=rng.randint(1,5)
    m=rng.randint(0,7)
    integer=[case%3==0 or (case%3==1 and rng.random()<0.5) for _ in range(n)]
    bounds=[(rng.randint(-3,0),rng.randint(1,4)) for _ in range(n)]
    cost=[rng.randint(-5,5) for _ in range(n)]
    offset=rng.randint(-4,4)
    maximize=rng.choice([False,True])
    rows=[([rng.randint(-4,4) for _ in range(n)],rng.randint(-10,10),rng.choice(['L','G','E'])) for _ in range(m)]
    lines=['NAME synthetic','OBJSENSE',' MAX' if maximize else ' MIN','ROWS',' N obj']
    lines += [f' {kind} r{i}' for i,(_,_,kind) in enumerate(rows)]
    lines+=['COLUMNS']
    for j in range(n):
        lines+=[f' x{j} obj {cost[j]}']
        lines += [f' x{j} r{i} {a[j]}' for i,(a,_,_) in enumerate(rows) if a[j]]
    lines+=['RHS',f' rhs obj {-offset}']
    lines += [f' rhs r{i} {b}' for i,(_,b,_) in enumerate(rows)]
    lines+=['BOUNDS']
    for j,(lo,up) in enumerate(bounds):
        lines += [f' {"LI" if integer[j] else "LO"} bounds x{j} {lo}', f' UP bounds x{j} {up}']
    lines+=['ENDATA']
    text='\n'.join(lines)+'\n'
    try:
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'case.mps';path.write_text(text)
            result=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'solve',str(path),'10'],timeout=15))
            h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('mip_rel_gap',0.);h.setOptionValue('mip_abs_gap',0.);h.readModel(str(path));h.run()
            status=h.getModelStatus()
            if status==highspy.HighsModelStatus.kInfeasible:
                assert result['status']=='infeasible', result
            else:
                assert status==highspy.HighsModelStatus.kOptimal, str(status)
                assert result['status']=='optimal', result
                expected=h.getObjectiveValue()
                assert abs(expected-result['objective'])<=1e-6*(1+abs(expected)),(expected,result)
                assert validate(path,result)['valid'], result
                if case==0:
                    (outdir/'example.mps').write_text(text)
                    (outdir/'example.solution.json').write_text(json.dumps(result,indent=2)+'\n')
            if all(integer):
                feasible=[]
                for x in itertools.product(*(range(lo,up+1) for lo,up in bounds)):
                    def satisfies(row):
                        a,b,kind=row;activity=sum(v*t for v,t in zip(a,x))
                        return activity<=b if kind=='L' else activity>=b if kind=='G' else activity==b
                    if all(map(satisfies,rows)):
                        feasible.append(offset+sum(v*t for v,t in zip(cost,x)))
                if not feasible: assert result['status']=='infeasible'
                else: assert abs((max(feasible) if maximize else min(feasible))-result['objective'])<1e-6
            print(json.dumps(dict(case=case,status='pass',solver_status=result['status'],nodes=result['nodes'],sha256=hashlib.sha256(text.encode()).hexdigest())),flush=True)
    except Exception as error:
        failed+=1
        (outdir/f'failure-{case}.mps').write_text(text)
        print(json.dumps(dict(case=case,status='fail',error=str(error))),flush=True)
print(json.dumps(dict(cases=count,failures=failed)),flush=True)
sys.exit(bool(failed))
