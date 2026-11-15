import os,json,subprocess,time
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
logs=[Path(f'artifacts/rational-certificate-smoke{n}.log')for n in [8,10]]+[Path('artifacts/rational-certificate-native.log'),Path('artifacts/rational-certificate-remaining-trace.log')]+list(Path('artifacts').glob('rational-certificate-repeat-*.log'))+[Path(f'artifacts/feasibility-pump-evaluation/feasibility-pump-first-smoke{n}.log')for n in [8,10]]
for p in logs:
 while not p.with_suffix('.exit').exists():time.sleep(1)
 assert p.with_suffix('.exit').read_text().strip()=='0'
runs=[Path(p.read_text().strip().splitlines()[-1])for p in logs];assert len(runs)==len(set(runs))
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/rational-certificate-audit',*map(str,runs)],check=True)
rows=[]
for group in [8,10]:
 def records(log):return{r['name']:r for r in map(json.loads,(Path(Path(log).read_text().strip().splitlines()[-1])/'results.jsonl').read_text().splitlines())}
 a=records(f'artifacts/feasibility-pump-evaluation/feasibility-pump-first-smoke{group}.log');b=records(f'artifacts/rational-certificate-smoke{group}.log')
 for n in a:
  keys=['status','objective','dual_bound'];x={k:a[n].get(k)for k in keys};y={k:b[n].get(k)for k in keys};rows.append(dict(name=n,before=x,after=y,same=x==y))
Path('artifacts/rational-certificate-comparison.json').write_text(json.dumps(rows,indent=2)+'\n');print('fixed same',sum(r['same']for r in rows),'/',len(rows),flush=True)
