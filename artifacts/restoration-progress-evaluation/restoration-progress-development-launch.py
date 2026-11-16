import json,subprocess,sys,time
from pathlib import Path
root=Path.cwd();cpu=int(sys.argv[1])
while True:
 p=root/'artifacts/restoration-progress-regression-exit.json'
 if p.exists():
  records=json.loads(p.read_text())
  if any(r['exit_code']for r in records):raise SystemExit('regression failed')
  if len(records)==6:break
 time.sleep(1)
if cpu in [1,2]:
 while not (root/f'artifacts/restoration-progress-mip-paired-{cpu}/exit.json').exists():time.sleep(1)
raise SystemExit(subprocess.call([str(root/'.venv/bin/python'),'artifacts/restoration-progress-development-screen.py',str(cpu),str(cpu),'3']))
