#!/usr/bin/env python3
"""低い専用cgroup上限で群全体のOOMを誘発し、kernelの終了証拠を確認。"""
import json
from pathlib import Path
import subprocess
import sys
import time
from benchmark_common import atomic_json

root=Path(__file__).resolve().parents[1]
path=root/'artifacts/oom-probe-config.json'
config=dict(id='runner-oom-probe',memory_bytes=64*1024**2,jobs=[dict(id='probe')])
atomic_json(path,config)
helper=root/'scripts/cgroup_control.py'
subprocess.run(['sudo','-n','/usr/bin/python3',str(helper),'setup',str(path)],check=True)
code='''import os,time
for i in range(4):
 p=os.fork()
 if p==0:
  data=bytearray(24*1024**2)
  time.sleep(10)
  os._exit(0)
time.sleep(10)
'''
started=time.monotonic()
with (root/'artifacts/oom-probe.log').open('w') as log:
    result=subprocess.run(['sudo','-n','/usr/bin/python3',str(helper),'exec',config['id'],'probe','0','1000','1000',
                           str(config['memory_bytes']),sys.executable,'-c',code],stdout=log,stderr=subprocess.STDOUT,timeout=10)
leaf=Path('/sys/fs/cgroup/mip-codex-bench')/config['id']/'probe'
events={k:int(v) for k,v in (line.split() for line in (leaf/'memory.events').read_text().splitlines())}
assert result.returncode!=0 and events['oom_kill']>0,events
atomic_json(root/'artifacts/oom-probe-result.json',dict(valid=True,exit_code=result.returncode,events=events,
            peak_bytes=int((leaf/'memory.peak').read_text()),seconds=time.monotonic()-started))
print('cgroup OOM verified')
