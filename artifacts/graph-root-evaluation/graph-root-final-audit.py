import json,os,subprocess,time
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
paths=[Path(f'artifacts/graph-root-final-{side}-smoke{group}.log')for side in ['before','after']for group in [8,10]]
end=time.monotonic()+600
while not all(p.with_suffix('.exit').exists()for p in paths):
 assert time.monotonic()<end,'smoke completion timeout'
 time.sleep(1)
runs=[]
for p in paths:
 assert p.with_suffix('.exit').read_text().strip()=='0',p
 runs.append(Path(p.read_text().strip().splitlines()[-1]));assert runs[-1].is_dir()
comparisons=[]
for group in [8,10]:
 a,b=[Path(f'artifacts/graph-root-final-{side}-smoke{group}.log').read_text().strip().splitlines()[-1]for side in ['before','after']]
 def records(p):return{r['name']:r for r in map(json.loads,Path(p,'results.jsonl').read_text().splitlines())}
 ra,rb=records(a),records(b)
 for name in ra:
  keys=['status','objective','dual_bound'];x={k:ra[name].get(k)for k in keys};y={k:rb[name].get(k)for k in keys}
  comparisons.append(dict(name=name,before=x,after=y,same=x==y));assert x==y,(name,x,y)
Path('artifacts/graph-root-comparison.json').write_text(json.dumps(comparisons,indent=2)+'\n')
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/graph-root-audit',*map(str,runs)],check=True)
