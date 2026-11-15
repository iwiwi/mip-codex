import os,json,time,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
runs=[];comparisons=[];end=time.monotonic()+600
for group in [8,10]:
 p=Path(f'artifacts/graph-prefix-after-smoke{group}.log')
 while not p.with_suffix('.exit').exists():
  assert time.monotonic()<end
  time.sleep(1)
 assert p.with_suffix('.exit').read_text().strip()=='0'
 a=Path(f'artifacts/graph-root-final-after-smoke{group}.log').read_text().strip().splitlines()[-1];b=p.read_text().strip().splitlines()[-1];runs.extend([a,b])
 def records(p):return{r['name']:r for r in map(json.loads,Path(p,'results.jsonl').read_text().splitlines())}
 ra,rb=records(a),records(b)
 for name in ra:
  keys=['status','objective','dual_bound'];x={k:ra[name].get(k)for k in keys};y={k:rb[name].get(k)for k in keys};comparisons.append(dict(name=name,before=x,after=y,same=x==y));assert x==y,(name,x,y)
Path('artifacts/graph-prefix-comparison.json').write_text(json.dumps(comparisons,indent=2)+'\n')
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/graph-prefix-audit',*runs],check=True)
