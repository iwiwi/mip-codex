#!/usr/bin/env python3
"""End-to-end synthetic records for cross-solver diagnostics, not solver measurements."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT/os.environ.get('MIP_BOUND_CONSISTENCY_DIR', 'artifacts/bound-consistency-fixtures')


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dump(value, raw=None):
    text = json.dumps(value)
    return text.replace('"RAW_DECIMAL"', raw) if raw else text


class Consistency(unittest.TestCase):
    def fixture(self, records, maximize=False, variant='one', run_name='run'):
        folder = OUT/self._testMethodName/run_name
        folder.mkdir(parents=True, exist_ok=True)
        model = folder/'toy.mps'
        model.write_text(f'NAME {variant}\nOBJSENSE\n {"MAX" if maximize else "MIN"}\nROWS\n N obj\nCOLUMNS\n x obj 1\nRHS\nBOUNDS\n LO b x 0\n UP b x 1e40\nENDATA\n')
        config = dict(instances=[dict(name='toy.mps', absolute_path=str(model.resolve()))],
                      fixture='synthetic metadata; not a solver measurement')
        (folder/'config.json').write_text(json.dumps(config))
        lines = []
        for k, record in enumerate(records):
            point = record.get('point')
            status = record.get('status', 'time_limit')
            d = folder/'cases'/str(k)
            d.mkdir(parents=True, exist_ok=True)
            result = dict(objective=point, values=None if point is None else dict(x=point),
                          dual_bound=record.get('dual'), status=status)
            (d/'result.json').write_text(dump(result, record.get('raw')))
            valid = record.get('valid', point is not None)
            summary = dict(job_id=str(k), name='toy.mps', input_sha256=digest(model),
                           solver='fixture-'+str(k), solver_status=status,
                           result_sha256=digest(d/'result.json'),
                           strict_validation=dict(valid=valid),
                           primal_bound=point if valid else None,
                           dual_bound=record.get('dual'))
            if record.get('unindexed'):
                summary.pop('result_sha256')
            if record.get('bad_hash'):
                summary['result_sha256'] = '0'*64
            lines.append(dump(summary, record.get('raw')))
        (folder/'results.jsonl').write_text('\n'.join(lines)+'\n')
        return folder

    def check_runs(self, runs, expect_failure=False):
        output = OUT/self._testMethodName/'diagnostic.json'
        command = [sys.executable, str(ROOT/'scripts/check_bound_consistency.py'),
                   *map(str, runs), '--output', str(output)]
        done = subprocess.run(command, capture_output=True, text=True)
        (output.parent/'command.json').write_text(json.dumps(command))
        (output.parent/'stdout.log').write_text(done.stdout)
        (output.parent/'stderr.log').write_text(done.stderr)
        (output.parent/'exit.json').write_text(json.dumps(dict(exit_code=done.returncode)))
        if expect_failure:
            self.assertNotEqual(done.returncode, 0)
            return done
        self.assertIn(done.returncode, (0, 2), done.stderr)
        report = json.loads(output.read_text())
        self.assertEqual(done.returncode, 2 if report['findings'] else 0)
        return report

    def test_minimum_cross_solver_contradiction(self):
        run = self.fixture([dict(point=2, dual=0), dict(point=4, dual=3)])
        report = self.check_runs([run])
        self.assertEqual(len(report['findings']), 1)
        self.assertEqual(report['findings'][0]['absolute_excess'], '1')
        self.assertIn('/cases/0/', report['findings'][0]['point_source'])

    def test_maximum_cross_solver_contradiction(self):
        run = self.fixture([dict(point=2, dual=3), dict(point=4, dual=10)], maximize=True)
        report = self.check_runs([run])
        self.assertEqual(len(report['findings']), 1)
        self.assertTrue(report['findings'][0]['maximize'])
        self.assertEqual(report['findings'][0]['absolute_excess'], '1')

    def test_invalid_point_is_excluded(self):
        run = self.fixture([dict(point=-100, dual=None, valid=False), dict(point=2, dual=1)])
        self.assertEqual(self.check_runs([run])['findings'], [])

    def test_infeasibility_is_flagged_for_point_review(self):
        run = self.fixture([dict(point=None, dual=None, status='infeasible'), dict(point=2, dual=0)])
        report = self.check_runs([run])
        self.assertEqual([r['type'] for r in report['findings']], ['infeasibility_claim_with_tolerance_checked_point'])

    def test_ambiguous_status_is_not_a_definite_infeasibility_claim(self):
        run = self.fixture([dict(point=None, dual=None, status='infeasible_or_unbounded'), dict(point=2, dual=0)])
        self.assertEqual(self.check_runs([run])['findings'], [])

    def test_exact_large_integer_is_not_rounded_to_binary64(self):
        value = 2**100
        run = self.fixture([dict(point=value, dual=value+1)])
        report = self.check_runs([run])
        self.assertEqual(report['findings'][0]['absolute_excess'], '1')

    def test_exact_tolerance_boundary_is_accepted(self):
        run = self.fixture([dict(point=2, dual='RAW_DECIMAL', raw='2.000001')])
        self.assertEqual(self.check_runs([run])['findings'], [])

    def test_decimal_just_outside_tolerance_is_detected(self):
        run = self.fixture([dict(point=2, dual='RAW_DECIMAL', raw='2.00000100000000000001')])
        self.assertEqual(len(self.check_runs([run])['findings']), 1)

    def test_different_input_hashes_are_not_combined(self):
        one = self.fixture([dict(point=10, dual=4)], variant='one', run_name='one')
        two = self.fixture([dict(point=2, dual=0)], variant='two', run_name='two')
        report = self.check_runs([one, two])
        self.assertEqual(report['input_groups'], 2)
        self.assertEqual(report['findings'], [])

    def test_partial_append_is_preserved_as_a_warning(self):
        run = self.fixture([dict(point=2, dual=1)])
        with (run/'results.jsonl').open('ab') as stream:
            stream.write(b'{"job')
        report = self.check_runs([run])
        self.assertEqual(len(report['warnings']), 1)
        self.assertEqual(report['sources'][0]['results_sha256'], digest(run/'results.jsonl'))
        self.assertFalse(report['sources'][0]['terminated_by_newline'])

    def test_missing_result_index_is_a_warning(self):
        run = self.fixture([dict(point=2, dual=1, unindexed=True)])
        self.assertEqual(len(self.check_runs([run])['warnings']), 1)

    def test_changed_result_is_rejected(self):
        run = self.fixture([dict(point=2, dual=1, bad_hash=True)])
        self.assertIn('result hash mismatch', self.check_runs([run], True).stderr)

    def test_duplicate_run_is_rejected(self):
        run = self.fixture([dict(point=2, dual=1)])
        self.assertIn('duplicate run directories', self.check_runs([run, run], True).stderr)


if __name__ == '__main__':
    unittest.main()
