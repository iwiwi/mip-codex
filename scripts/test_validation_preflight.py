#!/usr/bin/env python3
"""検査器未配置なら測定を開始せず、validator自体は未検証を返せる。"""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from validate_solution import absolute_checker
from benchmark_common import require_solver_origin
import json
root=Path(__file__).resolve().parents[1]
class Preflight(unittest.TestCase):
    def fake_binary(self, directory, source):
        path=Path(directory)/'solver'
        path.write_text('#!'+sys.executable+'\nprint('+repr(json.dumps(dict(source_directory=str(source),package_version='test')))+')\n')
        path.chmod(0o755)
        return path
    def test_binary_from_another_worktree_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary=self.fake_binary(tmp,root/'other-worktree')
            with self.assertRaisesRegex(RuntimeError,'solver binary was built in'):
                require_solver_origin(root,binary)
    def test_binary_from_the_selected_worktree_is_accepted(self):
        with tempfile.TemporaryDirectory() as tmp:
            binary=self.fake_binary(tmp,root)
            self.assertEqual(require_solver_origin(root,binary)['package_version'],'test')
    def test_explicit_checker_path_is_respected(self):
        with patch.dict(os.environ,{'MIP_ABSOLUTE_CHECKER':'/not-installed/checker'}):
            self.assertEqual(absolute_checker(),Path('/not-installed/checker'))
    def test_synthetic_missing_checker_stops_before_creating_outputs(self):
        with tempfile.TemporaryDirectory() as tmp:
            output=Path(tmp)/'not-created'
            env=dict(os.environ,MIP_ABSOLUTE_CHECKER=str(Path(tmp)/'missing'),MIP_SYNTHETIC_DIR=str(output))
            r=subprocess.run([sys.executable,str(root/'scripts/check_synthetic.py'),'0'],env=env,text=True,capture_output=True,timeout=15)
            self.assertNotEqual(r.returncode,0);self.assertIn('required absolute rational checker unavailable',r.stdout+r.stderr);self.assertFalse(output.exists())
    def test_smoke_missing_checker_stops_before_running_cases(self):
        with tempfile.TemporaryDirectory() as tmp:
            env=dict(os.environ,MIP_ABSOLUTE_CHECKER=str(Path(tmp)/'missing'))
            r=subprocess.run([sys.executable,str(root/'scripts/smoke_benchmark.py'),'0'],env=env,text=True,capture_output=True,timeout=15)
            self.assertNotEqual(r.returncode,0);self.assertIn('required absolute rational checker unavailable',r.stdout+r.stderr)
if __name__=='__main__':unittest.main()
