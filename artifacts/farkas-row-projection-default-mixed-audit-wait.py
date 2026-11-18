from pathlib import Path
import subprocess,time
root=Path.cwd();end=time.monotonic()+1800
while not(root/'artifacts/farkas-row-projection-default-synthetic/mixed-on/summary.json').exists():
 assert time.monotonic()<end;time.sleep(1)
with(root/'artifacts/farkas-row-projection-default-mixed-limitations-audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'artifacts/farkas-row-projection-default-mixed-limitations-audit.py'],stdout=f,stderr=subprocess.STDOUT,check=True)
