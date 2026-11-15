import json,os,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
labels=['initial-before','initial-after','before8','after8','before16','after16','long-before','long-after','native-timtab1','native-fhnw-binpack4-4','final-repeat']
runs=[Path(Path(f'artifacts/aggregate-cuts-{label}.log').read_text().strip().splitlines()[-1])for label in labels]
assert len(runs)==len(set(runs))
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/aggregate-cuts-final-smoke-audit',*map(str,runs)],check=True)
