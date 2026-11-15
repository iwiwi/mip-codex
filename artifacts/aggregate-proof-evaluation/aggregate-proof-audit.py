import json,os,subprocess,time
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
logs=[*Path('artifacts').glob('aggregate-proof-gen*.log'),Path('artifacts/aggregate-proof-smoke8.log'),Path('artifacts/aggregate-proof-smoke10.log'),*[Path(f'artifacts/joint-proof-evaluation/joint-proof-smoke{n}.log')for n in [8,10]]]
for p in logs:
 while not p.with_suffix('.exit').exists():time.sleep(1)
 assert p.with_suffix('.exit').read_text().strip()=='0'
runs=[str(Path(p.read_text().strip().splitlines()[-1])) for p in logs]
rows=[]
for g in [8,10]:
 def records(log):return{r['name']:r for r in map(json.loads,Path(Path(log).read_text().strip().splitlines()[-1],'results.jsonl').read_text().splitlines())}
 a=records(f'artifacts/joint-proof-evaluation/joint-proof-smoke{g}.log');b=records(f'artifacts/aggregate-proof-smoke{g}.log')
 for n in a:
  keys=['status','objective','dual_bound'];x={k:a[n].get(k)for k in keys};y={k:b[n].get(k)for k in keys};rows.append(dict(name=n,before=x,after=y,same=x==y))
Path('artifacts/aggregate-proof-comparison.json').write_text(json.dumps(rows,indent=2)+'\n')
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/aggregate-proof-audit',*runs],check=True)
