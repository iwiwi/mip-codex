#!/usr/bin/env python3
"""実プロセスでcoordinator中断・再開・完了後再開の重複実行を検査。"""
import json
from pathlib import Path
import subprocess
import sys
import time
from benchmark_common import group_alive, sha256

root=Path(__file__).resolve().parents[1]
experiment=root/'artifacts/runs'/('runner-resume-'+str(time.time_ns()))
subprocess.run([sys.executable,str(root/'scripts/benchmark.py'),'create',str(experiment),'--seconds','3',
                '--cores','0','--solvers','highs','--names','pk1.mps.gz','--allow-dirty'],check=True)
command=[sys.executable,str(experiment/'snapshot/scripts/benchmark.py'),'run',str(experiment)]
with (experiment/'coordinator-test.log').open('a') as log:
    first=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    launch_file=experiment/'cases/000-highs/solver-launch.json'
    deadline=time.monotonic()+20
    while not launch_file.exists():
        if first.poll() is not None or time.monotonic()>deadline:raise RuntimeError('solver did not launch')
        time.sleep(0.02)
    launch=json.loads(launch_file.read_text())
    assert group_alive(launch['pgid'])
    original_hash=sha256(launch_file)
    locked=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=10)
    assert locked.returncode!=0,'second coordinator must fail lock acquisition'
    first.terminate();first.wait(timeout=10)
    assert group_alive(launch['pgid']),'solver must remain live after coordinator termination'
    second=subprocess.Popen(command,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    assert second.wait(timeout=30)==0
    assert sha256(launch_file)==original_hash,'solver launch record must be unchanged'
    summary=experiment/'cases/000-highs/summary.json'
    original_summary=sha256(summary)
    assert json.loads(summary.read_text())['strict_validation']['valid'] is True
    assert subprocess.run(command,stdout=log,stderr=subprocess.STDOUT,timeout=15).returncode==0
    assert sha256(summary)==original_summary,'completed result must never be overwritten'
    assert sha256(launch_file)==original_hash
print(json.dumps(dict(valid=True,solver_pid_preserved=launch['pid'],launch_sha256=original_hash,
                      result_sha256=original_summary,completed_resume_did_not_restart=True)))
