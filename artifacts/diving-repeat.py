import os,subprocess
from pathlib import Path
root=Path(__file__).resolve().parent.parent
env=os.environ.copy();env['MIP_BENCH_SOLVERS']='rust-no-dive,rust-dive'
for trial in range(3):
 print('trial',trial,flush=True)
 subprocess.run(['taskset','-c','0',str(root/'.venv/bin/python'),str(root/'scripts/smoke_benchmark.py'),'5','gmu-35-40.mps.gz'],cwd=root,env=env,check=True)
