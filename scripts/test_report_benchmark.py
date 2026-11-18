import copy
import json
from pathlib import Path
import tempfile
import unittest
from report_benchmark import classify, combine, digest, report, pairwise


def point(status='optimal', objective=10, dual=10, seconds=2):
    return dict(status=status, primal_bound=objective, dual_bound=dual, seconds=seconds,
                strict_validation=dict(valid=True, exact=dict(objective=str(objective))),
                execution=dict(exit_code=0, watchdog=False, wall_seconds=3))


class ReportTests(unittest.TestCase):
    def test_exact_recomputed_primal_controls_gap(self):
        row = point(objective=9007199254740993, dual=9007199254740992)
        row['primal_bound'] = 9007199254740992.0
        r = classify(row, 3600)
        self.assertEqual(r['primal'], '9007199254740993')
        self.assertGreater(r['common_gap'], 0)
        self.assertTrue(r['declared_optimal_with_valid_primal'])
        self.assertFalse(r['mathematical_optimality_certificate_independently_verified'])

    def test_failures_and_unchecked_points_never_become_timed_wins(self):
        for status in ['watchdog', 'process_error', 'oom', 'error', 'supervisor_crash']:
            r = classify(point(status), 3600)
            self.assertIsNone(r['primal'])
            self.assertIsNone(r['dual'])
            self.assertEqual(r['charged_seconds'], 3600)
        row = point()
        row['strict_validation']['valid'] = False
        r = classify(row, 3600)
        self.assertIsNone(r['primal'])
        self.assertFalse(r['declared_optimal_with_valid_primal'])
        row = point()
        row['execution']['watchdog'] = True
        self.assertIsNone(classify(row, 3600)['primal'])

    def test_time_limit_points_count_but_pay_full_unsolved_penalty(self):
        r = classify(point('time_limit', dual=5, seconds=1), 3600)
        self.assertEqual(r['primal'], '10')
        self.assertEqual(r['common_gap'], 0.5)
        self.assertEqual(r['charged_seconds'], 3600)
        self.assertFalse(r['declared_optimal_with_valid_primal'])
        self.assertEqual(classify({'status':'missing'}, 3600)['charged_seconds'], 3600)

    def test_wrong_direction_dual_cannot_score_as_a_strong_bound(self):
        for maximize, bad in [(False, 12), (True, 8)]:
            r = classify(point(dual=bad), 3600, maximize)
            self.assertTrue(r['signed_bound_inconsistency'])
            self.assertIsNone(r['dual'])
            self.assertIsNone(r['common_gap'])
        row = point()
        row['cut_validation'] = dict(valid=False)
        self.assertIsNone(classify(row, 3600)['dual'])
        self.assertEqual(classify(row, 3600)['primal'], '10')
        for bad in [float('nan'), float('inf'), '-inf', True]:
            self.assertIsNone(classify(point(dual=bad), 3600)['dual'])

    def test_pairwise_senses_missing_values_and_fractional_ties(self):
        cases = [dict(name='x',solver='a',primal='3',dual='1'), dict(name='x',solver='b',primal='4',dual='2'),
                 dict(name='y',solver='a',primal=None,dual='0'), dict(name='y',solver='b',primal='0',dual=None)]
        r = pairwise(cases)[0]
        self.assertEqual(r['primal'], {'left_better':1,'right_only':1})
        self.assertEqual(r['dual'], {'right_better':1,'left_only':1})
        r = pairwise(cases,True)[0]
        self.assertEqual(r['primal']['right_better'], 1)
        self.assertEqual(r['dual']['left_better'], 1)

    def test_shifted_mean_penalties_and_duplicate_replacements(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.make_run(root/'base', point())
            self.make_run(root/'new', point())
            with self.assertRaisesRegex(ValueError, 'multiple replacements'):
                combine([root/'base'], [root/'new',root/'new'])
            resources, rows, sources, originals = combine([root/'base'])
            r = report(resources, rows, sources, originals)
            self.assertAlmostEqual(r['summary']['own']['shifted_geometric_seconds_10'], 2)
            row = point()
            del row['execution']
            self.assertFalse(classify(row,3600)['declared_optimal_with_valid_primal'])
            self.assertIsNone(classify(row,3600)['primal'])

    def make_run(self, root, row, cpu=8, seconds=3600):
        root.mkdir()
        instance = dict(name='synthetic', sha256='a'*64, split='development')
        config = dict(instances=[instance], jobs=[dict(id='000-own', instance_index=0, solver='own', cpu=cpu)],
                      seconds=seconds, memory_bytes=16384, threads=1, cores=[8], read_write_grace_seconds=60,
                      validation_seconds=600, manifest_sha256='b'*64)
        (root/'config.json').write_text(json.dumps(config))
        if row is not None:
            d = root/'cases/000-own'
            d.mkdir(parents=True)
            (d/'result.json').write_text('{}')
            row = dict(row, name='synthetic', solver='own', input_sha256=instance['sha256'], result_sha256=digest(d/'result.json'))
            (d/'summary.json').write_text(json.dumps(row))

    def test_predetermined_replacement_keeps_worse_result_and_original(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.make_run(root/'original', point())
            self.make_run(root/'replacement', point('time_limit', objective=20, dual=5))
            resources, rows, sources, originals = combine([root/'original'], [root/'replacement'])
            self.assertEqual(rows[('synthetic','own')]['primal_bound'], 20)
            self.assertEqual(originals[0]['primal_bound'], 10)
            r = report(resources, rows, sources, originals)
            self.assertAlmostEqual(r['summary']['own']['shifted_geometric_seconds_10'], 3600)
            self.assertEqual(r['summary']['own']['strict_feasible'], 1)
            with self.assertRaises(ValueError):
                combine([root/'original', root/'replacement'])
            (root/'replacement/cases/000-own/result.json').write_text('{"tampered":true}')
            with self.assertRaisesRegex(ValueError, 'hash mismatch'):
                combine([root/'original'], [root/'replacement'])

    def test_resource_or_cpu_mismatch_rejected_and_missing_retained(self):
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            self.make_run(root/'base', point())
            self.make_run(root/'cpu', point(), cpu=9)
            self.make_run(root/'time', point(), seconds=60)
            for other in ['cpu','time']:
                with self.assertRaises(ValueError):
                    combine([root/'base'], [root/other])
            self.make_run(root/'missing', None)
            resources, rows, sources, originals = combine([root/'missing'])
            r = report(resources, rows, sources, originals)
            self.assertFalse(r['complete'])
            self.assertEqual(r['summary']['own']['statuses'], {'missing':1})
            self.assertEqual(r['summary']['own']['strict_feasible'], 0)


if __name__ == '__main__':
    unittest.main()
