import os,subprocess
from pathlib import Path
os.sched_setaffinity(0,set(range(33,42)))
logs=[Path('artifacts/ratio-residual-diagnostic-run.log'),Path('artifacts/basis-refinement-candidate-diagnostic-run.log'),Path('artifacts/basis-refinement-candidate-original-run.log')]
runs=[Path(p.read_text().strip().splitlines()[-1])for p in logs]
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/basis-refinement-audit',*map(str,runs)],check=True)
