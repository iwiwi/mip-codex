import json,os,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
logs=[*Path('artifacts').glob('joint-proof-gen*.log'),Path('artifacts/joint-proof-smoke8.log'),Path('artifacts/joint-proof-smoke10.log'),Path('artifacts/joint-proof-dump-run.log'),*[Path(f'artifacts/defer-infeasibility-evaluation/defer-infeasibility-smoke{n}.log')for n in [8,10]]]
runs=[str(Path(p.read_text().strip().splitlines()[-1])) for p in logs]
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/joint-proof-audit',*runs],check=True)
