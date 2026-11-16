#!/usr/bin/env python3
"""独自Rustの費用変更warmをcold/dense回帰に加えてHiGHSへ独立照合する。"""
import json,os,random,subprocess,time,hashlib
from pathlib import Path
from fractions import Fraction
import numpy as np
import highspy
root=Path(__file__).resolve().parents[1]
out=root/os.environ.get('MIP_COST_DIR','artifacts/warm-objective-cost-series');out.mkdir(parents=True,exist_ok=True)
rng=random.Random(2026111514);cases=[]
for case in range(100):
    n=rng.randint(24,48);m=rng.randint(8,20);upper=[2*rng.randint(1,4) for _ in range(n)]
    matrix=[[rng.randint(-3,3) if rng.random()<.35 else 0 for _ in range(n)] for _ in range(m)]
    equality=[rng.randrange(4)==0 for _ in range(m)]
    rhs=[sum(a*u//2 for a,u in zip(row,upper))+(0 if eq else rng.randint(0,4)) for row,eq in zip(matrix,equality)]
    columns=[[(i,row[j]) for i,row in enumerate(matrix) if row[j]] for j in range(n)]
    costs=[[rng.randint(-10,10) for _ in range(n)] for _ in range(11)]
    cases.append(dict(columns=columns,rhs=rhs,upper=upper,equality=equality,costs=costs))
input_path=out/'input.json';input_path.write_text(json.dumps(cases)+'\n')
binary=Path(os.environ.get('MIP_COST_BIN',root/'target/release/examples/reopt_lp'))
meta=dict(seed=2026111514,models=100,changes_per_model=10,cpu=sorted(os.sched_getaffinity(0)),highs_version=highspy.Highs().version(),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),input_sha256=hashlib.sha256(input_path.read_bytes()).hexdigest(),git_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),source_sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'src/revised.rs',root/'examples/reopt_lp.rs',Path(__file__)]})
(out/'experiment.json').write_text(json.dumps(meta,indent=2)+'\n')
with input_path.open() as inp,(out/'rust.jsonl').open('w') as stream,(out/'stderr.log').open('w') as err:
    completed=subprocess.run([str(binary)],stdin=inp,stdout=stream,stderr=err,timeout=180)
assert completed.returncode==0,completed.returncode
rust=[json.loads(line) for line in (out/'rust.jsonl').read_text().splitlines()];assert len(rust)==1100
failed=0;warm_seconds=cold_seconds=highs_seconds=0.;warm_pivots=cold_pivots=successes=0
with (out/'validation.jsonl').open('w') as validation:
 for case,p in enumerate(cases):
    n=len(p['upper']);m=len(p['rhs']);h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1)
    h.addVars(n,np.zeros(n),np.array(p['upper'],dtype=float))
    rows=[[] for _ in range(m)]
    for j,column in enumerate(p['columns']):
        for i,a in column:rows[i].append((j,a))
    starts=[0];indices=[];values=[]
    for row in rows:
        for j,a in row:indices.append(j);values.append(a)
        starts.append(len(indices))
    h.addRows(m,np.array([b if eq else -highspy.kHighsInf for b,eq in zip(p['rhs'],p['equality'])]),np.array(p['rhs'],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(values,dtype=float))
    for step,cost in enumerate(p['costs']):
        r=rust[case*11+step];assert (r['case'],r['step'])==(case,step)
        h.changeColsCost(n,np.arange(n,dtype=np.int32),np.array(cost,dtype=float));begin=time.monotonic();h.run();elapsed=time.monotonic()-begin
        try:
            assert h.getModelStatus()==highspy.HighsModelStatus.kOptimal,str(h.getModelStatus())
            for mode in ['warm','cold']:
                result=r[mode];assert result['status']=='optimal',result
                x=list(map(lambda x:Fraction(str(x)),result['x']));tol=Fraction(1,1000000)
                assert len(x)==n and all(-tol<=v<=u+tol for v,u in zip(x,p['upper']))
                for row,b,eq in zip(rows,p['rhs'],p['equality']):
                    activity=sum(a*x[j] for j,a in row)
                    assert abs(activity-b)<=tol if eq else activity<=b+tol
                objective=sum(c*v for c,v in zip(cost,x));assert abs(objective-Fraction(str(result['objective'])))<=tol
                assert abs(float(objective)-h.getObjectiveValue())<=1e-7*(1+abs(h.getObjectiveValue()))
            status=dict(case=case,step=step,status='pass',highs_objective=h.getObjectiveValue(),highs_seconds=elapsed)
        except Exception as e:
            failed+=1;status=dict(case=case,step=step,status='fail',error=str(e))
        validation.write(json.dumps(status)+'\n')
        if step:
            warm_seconds+=r['warm_seconds'];cold_seconds+=r['cold_seconds'];highs_seconds+=elapsed;warm_pivots+=r['warm_pivots'];cold_pivots+=r['cold_pivots'];successes+=r['warm_cost_successes']
summary=dict(models=100,reoptimizations=1000,checked=1100,failures=failed,warm_seconds=warm_seconds,cold_seconds=cold_seconds,highs_warm_seconds=highs_seconds,warm_pivots=warm_pivots,cold_pivots=cold_pivots,warm_cost_successes=successes)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(failed))
