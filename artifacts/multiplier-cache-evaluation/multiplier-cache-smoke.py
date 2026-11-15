import json,os,subprocess,sys
from pathlib import Path
old=Path(f'artifacts/exact-point-smoke{sys.argv[1]}.log').read_text().strip().splitlines()[-1];m=json.loads(Path(old,'experiment.json').read_text());raise SystemExit(subprocess.run(['.venv/bin/python','scripts/smoke_benchmark.py',str(m['seconds']),*m['names']],env=dict(os.environ,MIP_BENCH_SOLVERS='rust')).returncode)
