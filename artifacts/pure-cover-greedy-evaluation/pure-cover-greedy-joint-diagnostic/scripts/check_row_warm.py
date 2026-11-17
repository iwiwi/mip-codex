#!/usr/bin/env python3
"""行追加warm/coldを独立したHiGHS照合と原単位Fraction検査で比較。"""
import hashlib,json,os,random,subprocess,time
from pathlib import Path
from fractions import Fraction
import numpy as np
import highspy
root=Path(__file__).resolve().parents[1]
out=root/os.environ.get('MIP_ROW_DIR','artifacts/warm-row-extension-series');out.mkdir(parents=True,exist_ok=True)
rng=random.Random(2026111601);cases=[]
for case in range(100):
 n=rng.randint(24,48);base=rng.randint(8,20);m=base+10;upper=[2*rng.randint(1,4) for _ in range(n)]
 matrix=[[rng.randint(-3,3) if rng.random()<.35 else 0 for _ in range(n)] for _ in range(m)]
 equality=[r<base and rng.randrange(4)==0 for r in range(m)]
 rhs=[sum(a*u//2 for a,u in zip(row,upper))+(0 if eq else rng.randint(0,4)) for row,eq in zip(matrix,equality)]
 cases.append(dict(columns=[[(r,row[j]) for r,row in enumerate(matrix) if row[j]] for j in range(n)],rhs=rhs,upper=upper,equality=equality,costs=[rng.randint(-10,10) for _ in range(n)],base_rows=base))
input_path=out/'input.json';input_path.write_text(json.dumps(cases)+'\n')
binary=Path(os.environ.get('MIP_ROW_BIN',root/'target/release/examples/append_lp'))
meta=dict(seed=2026111601,models=100,changes_per_model=10,cpu=sorted(os.sched_getaffinity(0)),highs_version=highspy.Highs().version(),binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),input_sha256=hashlib.sha256(input_path.read_bytes()).hexdigest(),git_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),source_sha256={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [root/'src/revised.rs',root/'examples/append_lp.rs',Path(__file__)]})
(out/'experiment.json').write_text(json.dumps(meta,indent=2)+'\n')
with input_path.open() as inp,(out/'rust.jsonl').open('w') as stream,(out/'stderr.log').open('w') as err:
 completed=subprocess.run([str(binary)],stdin=inp,stdout=stream,stderr=err,timeout=300)
assert completed.returncode==0,completed.returncode
rust=[json.loads(line) for line in (out/'rust.jsonl').read_text().splitlines()];assert len(rust)==1100
failed=0;totals=dict(warm_seconds=0.,cold_seconds=0.,highs_warm_seconds=0.,warm_pivots=0,cold_pivots=0,warm_row_extensions=0,warm_successes=0,warm_fallbacks=0)
with (out/'validation.jsonl').open('w') as validation:
 for case,p in enumerate(cases):
  n=len(p['upper']);rows=[[] for _ in p['rhs']]
  for j,col in enumerate(p['columns']):
   for r,a in col:rows[r].append((j,a))
  h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1)
  h.addVars(n,np.zeros(n),np.array(p['upper'],dtype=float));h.changeColsCost(n,np.arange(n,dtype=np.int32),np.array(p['costs'],dtype=float))
  def addrow(row,b,eq):
   h.addRow(b if eq else -highspy.kHighsInf,b,len(row),np.array([j for j,a in row],dtype=np.int32),np.array([a for j,a in row],dtype=float))
  for r in range(p['base_rows']):addrow(rows[r],p['rhs'][r],p['equality'][r])
  for step in range(11):
   count=p['base_rows']+step;r=rust[case*11+step];assert (r['case'],r['step'])==(case,step)
   if step:addrow(rows[count-1],p['rhs'][count-1],False)
   begin=time.monotonic();h.run();elapsed=time.monotonic()-begin
   try:
    assert h.getModelStatus()==highspy.HighsModelStatus.kOptimal,str(h.getModelStatus())
    for mode in ['warm','cold']:
     result=r[mode];assert result['status']=='optimal',result
     x=[Fraction(str(v)) for v in result['x']];tol=Fraction(1,1000000)
     assert len(x)==n and all(-tol<=v<=u+tol for v,u in zip(x,p['upper']))
     for row,b,eq in zip(rows[:count],p['rhs'],p['equality']):
      activity=sum(a*x[j] for j,a in row)
      assert abs(activity-b)<=tol if eq else activity<=b+tol
     objective=sum(c*v for c,v in zip(p['costs'],x));assert abs(objective-Fraction(str(result['objective'])))<=tol
     assert abs(float(objective)-h.getObjectiveValue())<=1e-7*(1+abs(h.getObjectiveValue()))
    status=dict(case=case,step=step,status='pass',highs_objective=h.getObjectiveValue(),highs_seconds=elapsed)
   except Exception as e:
    failed+=1;status=dict(case=case,step=step,status='fail',error=str(e))
   validation.write(json.dumps(status)+'\n')
   if step:
    for key in totals:
     totals[key]+=elapsed if key=='highs_warm_seconds' else r[key]
summary=dict(models=100,reoptimizations=1000,checked=1100,failures=failed,**totals)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(failed))
