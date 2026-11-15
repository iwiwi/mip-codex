import json,os,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
logs=[Path(f'artifacts/fractional-cover-smoke{n}.log')for n in [8,10]]
runs=[Path(p.read_text().strip().splitlines()[-1])for p in logs]
assert len(runs)==len(set(runs))
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/fractional-cover-smoke-audit',*map(str,runs)],check=True)
rows=[]
for n in [8,10]:
 def records(log):return{r['name']:r for r in map(json.loads,Path(Path(log).read_text().strip().splitlines()[-1],'results.jsonl').read_text().splitlines())}
 a=records(f'artifacts/graph-cover-smoke{n}.log');b=records(f'artifacts/fractional-cover-smoke{n}.log')
 for name in a:
  keys=['status','objective','dual_bound'];x={k:a[name].get(k)for k in keys};y={k:b[name].get(k)for k in keys};rows.append(dict(name=name,before=x,after=y,same=x==y))
Path('artifacts/fractional-cover-comparison.json').write_text(json.dumps(rows,indent=2)+'\n')
