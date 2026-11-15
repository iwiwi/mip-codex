import json,os,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
patterns=['feasibility-pump-smoke*.log','feasibility-pump-first-smoke*.log','feasibility-pump-repeat-*.log','feasibility-pump-native-*.log','feasibility-pump-extra-*.log','feasibility-pump-blpic-repeat-*.log','feasibility-pump-candidate.log']
logs=[p for pattern in patterns for p in Path('artifacts').glob(pattern)]
logs += [Path(f'artifacts/aggregate-proof-evaluation/aggregate-proof-smoke{n}.log')for n in [8,10]]
runs=[]
for p in logs:
 assert p.with_suffix('.exit').read_text().strip()=='0',p
 d=Path(p.read_text().strip().splitlines()[-1]);assert(d/'experiment.json').is_file();runs.append(str(d))
assert len(runs)==len(set(runs))
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/feasibility-pump-audit',*runs],check=True)
