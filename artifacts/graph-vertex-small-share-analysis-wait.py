from pathlib import Path
import time,subprocess,json,os
os.sched_setaffinity(0,{38});root=Path.cwd();audit=root/'artifacts/graph-vertex-small-share-development-parallel-audit'
while not(audit/'exit.json').exists():
 failures=[p for p in audit.glob('worker-*-exit.json')if json.load(open(p))['status']!='complete']
 assert not failures,failures
 time.sleep(1)
assert json.load(open(audit/'exit.json'))['status']=='complete'
with(root/'artifacts/graph-vertex-small-share-development-analysis.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'artifacts/graph-vertex-small-share-development-analysis.py'],stdout=f,stderr=subprocess.STDOUT,check=True)
