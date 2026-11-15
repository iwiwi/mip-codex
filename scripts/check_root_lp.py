#!/usr/bin/env python3
"""中規模の根LPをdense/revised/HiGHSで照合。範囲・等式・固定変数を含む。"""
import json
import os
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import highspy

os.environ['MIP_PRIMAL_REPAIR'] = '0'  # isolate LP engine work
os.environ['MIP_PROPAGATION'] = '0'  # compare original continuous LP relaxations
root=Path(__file__).resolve().parents[1]
rng=random.Random(2026111502)
count=int(sys.argv[1]) if len(sys.argv)>1 else 100
failed=0
profile=os.environ.get('MIP_ROOT_PROFILE','ordinary')
assert profile in ('ordinary','scaled','units')
engines=os.environ.get('MIP_ROOT_ENGINES','dense,revised').split(',')
assert engines and all(e in ('dense','revised') for e in engines)
outdir=Path(os.environ.get('MIP_ROOT_DIR',root/'artifacts'))
outdir.mkdir(parents=True,exist_ok=True)
def mps_text(rows,bounds,costs,integers):
    n=len(costs)
    lines=['NAME root','ROWS',' N obj']+[f' {kind} r{i}' for i,(_,_,kind,_) in enumerate(rows)]+['COLUMNS']
    for j in range(n):
        lines+=[f' x{j} obj {costs[j]}']+[f' x{j} r{i} {a[j]}' for i,(a,_,_,_) in enumerate(rows) if a[j]]
    lines+=['RHS']+[f' rhs r{i} {rhs}' for i,(_,rhs,_,_) in enumerate(rows)]
    if any(width is not None for _,_,_,width in rows):
        lines+=['RANGES']+[f' ran r{i} {width}' for i,(_,_,_,width) in enumerate(rows) if width is not None]
    lines+=['BOUNDS']
    for j,(lo,up) in enumerate(bounds):lines+=[f' {"LI" if integers[j] else "LO"} b x{j} {lo}',f' UP b x{j} {up}']
    lines+=['ENDATA']
    return '\n'.join(lines)+'\n'

with tempfile.TemporaryDirectory() as directory:
    path=Path(directory)/'root.mps'
    for case in range(count):
        n=rng.randint(15,60);m=rng.randint(10,70)
        bounds=[(rng.randint(-2,0),rng.randint(1,4)) for _ in range(n)]
        planted=[rng.randint(lo,up) for lo,up in bounds]
        integers=[rng.random()<0.3 for _ in range(n)]
        costs=[rng.randint(-10,10)/rng.choice([1,1,10]) for _ in range(n)]
        rows=[]
        for i in range(m):
            a=[rng.randint(-7,7) if rng.random()<0.3 else 0 for _ in range(n)]
            activity=sum(v*x for v,x in zip(a,planted))
            kind=rng.choice(['E','L','G','L','L'])
            rhs=activity+(rng.randint(0,15)*(1 if kind=='L' else -1) if kind!='E' else 0)
            width=rng.randint(0,20) if kind=='L' and rng.random()<0.25 else None
            if width is not None:width=max(width,rhs-activity)
            rows.append((a,rhs,kind,width))
        cost_scale=1.
        if profile=='scaled':
            transformed=[]
            for a,b,kind,width in rows:
                scale=rng.choice([1e-3,1e-2,1.,1e2,1e3])
                transformed.append(([v*scale for v in a],b*scale,kind,None if width is None else width*scale))
            rows=transformed
            cost_scale=[1e-10,1.,1e8][case%3]
            costs=[v*cost_scale for v in costs]
        reference_path=path
        if profile=='units':
            reference_path=Path(directory)/'reference.mps'
            reference_path.write_text(mps_text(rows,bounds,costs,[False]*n))
            units=[2.**rng.choice([-30,-15,0,15,30]) for _ in range(n)]
            transformed=[]
            for a,b,kind,width in rows:
                scale=2.**rng.choice([-12,-6,0,6,12])
                transformed.append(([v*units[j]*scale for j,v in enumerate(a)],b*scale,kind,None if width is None else width*scale))
            rows=transformed
            costs=[v*units[j] for j,v in enumerate(costs)]
            bounds=[(lo/units[j],up/units[j]) for j,(lo,up) in enumerate(bounds)]
            integers=[False]*n
        text=mps_text(rows,bounds,costs,integers)
        path.write_text(text)
        try:
            h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.readModel(str(reference_path))
            lp=h.getLp();lp.integrality_=[]
            # 正の目的スケールは最適点を変えない。参照側は単位を戻して小さい係数の無視を防ぐ。
            lp.col_cost_=[v/cost_scale for v in lp.col_cost_]
            h.passModel(lp);h.run()
            assert h.getModelStatus()==highspy.HighsModelStatus.kOptimal
            expected=h.getObjectiveValue()*cost_scale
            results={}
            for engine in engines:
                env=os.environ.copy();env['MIP_LP_ENGINE']=engine
                result=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'solve',str(path),'10','1'],env=env,timeout=15))
                assert result['status'] in ('optimal','node_limit'),result
                actual=result['dual_bound'];assert actual is not None,result
                assert abs(actual-expected)/cost_scale<=1e-6*(1+abs(expected/cost_scale)),(expected,result)
                if profile=='units':
                    from validate_solution import validate
                    assert validate(path,result)['valid'], 'original-MPS LP point invalid'
                results[engine]=dict(bound=actual,seconds=result['seconds'],pivots=result['lp_pivots'],refactors=result['lp_refactors'],scaled_columns=result.get('lp_scaled_columns'),scaled_rows=result.get('lp_scaled_rows'))
            print(json.dumps(dict(case=case,status='pass',profile=profile,cost_scale=cost_scale,highs=expected,results=results)),flush=True)
        except Exception as error:
            failed+=1
            (outdir/f'root-{profile}-failure-{case}.mps').write_text(text)
            print(json.dumps(dict(case=case,status='fail',error=str(error))),flush=True)
print(json.dumps(dict(cases=count,failures=failed)),flush=True)
sys.exit(bool(failed))
