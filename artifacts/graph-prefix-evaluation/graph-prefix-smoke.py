import json,os,subprocess,sys
from pathlib import Path
side=sys.argv[1];group=sys.argv[2];enabled=int(side=='after');old=Path(f'artifacts/graph-local-before-smoke{group}.log').read_text().strip().splitlines()[-1];m=json.loads(Path(old,'experiment.json').read_text());cmd=['.venv/bin/python','scripts/smoke_benchmark.py',str(m['seconds']),*m['names']]
p=subprocess.run(cmd,env=dict(os.environ,MIP_BENCH_SOLVERS='rust',MIP_GRAPH_ROOT_COLORING='1',MIP_GRAPH_ROOT_PREFIX='1'),check=False);raise SystemExit(p.returncode)
