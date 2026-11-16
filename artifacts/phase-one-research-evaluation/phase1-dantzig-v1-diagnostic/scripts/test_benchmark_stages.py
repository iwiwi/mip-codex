#!/usr/bin/env python3
"""watchdogと生存プロセスの回収が最適化を再起動しないことを検査。"""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest
from benchmark_case import stage
from benchmark_common import atomic_json, group_alive, process

class Stages(unittest.TestCase):
    def test_watchdog_and_idempotence(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            command=[sys.executable,'-c','import time; print("started",flush=True); time.sleep(30)']
            first=stage(root,'test',command,0.2)
            launch=json.loads((root/'test-launch.json').read_text())
            self.assertTrue(first['watchdog'])
            self.assertFalse(group_alive(launch['pgid']))
            second=stage(root,'test',command,0.2)
            self.assertEqual(first,second)
            self.assertEqual((root/'test-stdout.log').read_text().count('started'),1)
    def test_recover_live_stage_preserves_pid(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            command=[sys.executable,'-c','import time; time.sleep(0.6); print("done")']
            with (root/'test-stdout.log').open('w') as stdout:
                child=subprocess.Popen(['/usr/bin/time','-v','-o',str(root/'test-resource.log'),*command],stdout=stdout,start_new_session=True)
            identity=process(child.pid)
            atomic_json(root/'test-launch.json',dict(command=command,pid=child.pid,pgid=child.pid,start_ticks=identity['start_ticks'],
                        started_monotonic=time.monotonic(),deadline_monotonic=time.monotonic()+3,
                        boot_id=Path('/proc/sys/kernel/random/boot_id').read_text().strip()))
            result=stage(root,'test',command,99)
            self.assertTrue(result['recovered']);self.assertFalse(result['watchdog'])
            self.assertEqual(json.loads((root/'test-launch.json').read_text())['pid'],child.pid)
            self.assertEqual((root/'test-stdout.log').read_text().strip(),'done')
            self.assertEqual(child.wait(),0)
if __name__=='__main__':unittest.main()
