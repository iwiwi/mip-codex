#!/usr/bin/env python3
"""Actual GMP checks of the documented relative objective policy; no benchmark claims."""
import json
import os
from pathlib import Path
import subprocess
import unittest
from fractions import Fraction
from benchmark_common import require_solver_origin, atomic_json, sha256
from validate_solution import validate

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/os.environ.get('MIP_OBJECTIVE_EVALUATION_DIR', 'artifacts/canonical-objective-rational')


class ObjectivePolicy(unittest.TestCase):
    def test_large_objective_reports_rounding_and_exact_recomputed_value(self):
        d = OUT/'large'
        d.mkdir(parents=True, exist_ok=True)
        coefficient = -(2**52-1)
        model = d/'model.mps'
        model.write_text('NAME large\nROWS\n N obj\nCOLUMNS\n'+''.join(f' {v} obj {coefficient}\n' for v in ['x','y','z'])+'RHS\nBOUNDS\n'+''.join(f' BV b {v}\n' for v in ['x','y','z'])+'ENDATA\n')
        binary = ROOT/'target/release/mip-codex'
        info = require_solver_origin(ROOT, binary)
        command = [str(binary), 'solve', str(model), '5']
        done = subprocess.run(command, capture_output=True, text=True, timeout=15)
        (d/'result.json').write_text(done.stdout)
        (d/'stderr.log').write_text(done.stderr)
        atomic_json(d/'receipt.json', dict(command=command, build_info=info,
                                         binary_sha256=sha256(binary), input_sha256=sha256(model),
                                         exit_code=done.returncode, kind='numeric regression not benchmark'))
        self.assertEqual(done.returncode, 0)
        result = json.loads(done.stdout)
        self.assertEqual(result['status'], 'optimal')
        self.assertEqual(list(result['values'].values()), [1.,1.,1.])
        checked = validate(model, result)
        atomic_json(d/'validation.json', checked)
        self.assertTrue(checked['valid'])
        self.assertEqual(checked['exact']['objective'], str(3*coefficient))
        self.assertEqual(checked['exact']['objective_error'], '1')
        self.assertEqual(checked['objective_evaluation']['criterion'], 'relative')
        self.assertIn('relative objective', checked['arithmetic'])

    def test_small_absolute_error_can_exceed_relative_objective_tolerance(self):
        d = OUT/'near-one'
        d.mkdir(parents=True, exist_ok=True)
        model = d/'model.mps'
        model.write_text('NAME near-one\nROWS\n N obj\nCOLUMNS\n x obj 1\nRHS\nBOUNDS\n FX b x 1\nENDATA\n')
        point = dict(objective=1.0000001, values=dict(x=1))
        atomic_json(d/'point.json', point)
        checked = validate(model, point)
        atomic_json(d/'validation.json', checked)
        self.assertFalse(checked['valid'])
        self.assertEqual(Fraction(checked['objective_evaluation']['absolute_error']), Fraction(1,10**7))
        self.assertEqual(Fraction(checked['objective_evaluation']['absolute_tolerance_at_point']), Fraction(1,5*10**7))


if __name__ == '__main__':
    unittest.main()
