#!/usr/bin/env python3
"""Read-only weak-duality diagnostics; this does not certify reported dual bounds."""
import argparse
import hashlib
import json
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
from benchmark_common import atomic_json, sha256, utc
from verify_binary_graph import maximize_sense


def number(value):
    if value is None:
        return None
    if type(value) is int:
        return Fraction(value)
    if isinstance(value, Decimal) and value.is_finite():
        return Fraction(value)
    raise ValueError('nonfinite or nonnumeric reported bound/objective')


def collect(runs):
    groups, sources, warnings, inputs = {}, [], [], {}
    for run in runs:
        config_bytes = (run/'config.json').read_bytes()
        config = json.loads(config_bytes)
        models = {r['name']: r for r in config['instances']}
        payload = (run/'results.jsonl').read_bytes()
        sources.append(dict(run=str(run.resolve()),
                            results_sha256=hashlib.sha256(payload).hexdigest(),
                            results_bytes=len(payload),
                            terminated_by_newline=payload.endswith(b'\n'),
                            config_sha256=hashlib.sha256(config_bytes).hexdigest()))
        for index, line in enumerate(payload.splitlines()):
            try:
                summary = json.loads(line, parse_float=Decimal)
            except json.JSONDecodeError:
                warnings.append(dict(run=str(run), line=index+1,
                                     error='incomplete_or_invalid_json_record'))
                continue
            model = models[summary['name']]
            path = Path(model['absolute_path'])
            if path not in inputs:
                inputs[path] = (sha256(path), maximize_sense(path))
            input_sha, maximize = inputs[path]
            if input_sha != summary['input_sha256']:
                raise ValueError('original input hash mismatch: '+str(path))
            key = (summary['name'], input_sha)
            group = groups.setdefault(key, dict(maximize=maximize, records=[]))
            if group['maximize'] != maximize:
                raise ValueError('objective sense mismatch for identical input')
            result = run/'cases'/summary['job_id']/'result.json'
            expected = summary.get('result_sha256')
            if not result.exists() or expected is None:
                warnings.append(dict(run=str(run), job=summary['job_id'],
                                     error='missing_or_unindexed_result'))
                continue
            result_bytes = result.read_bytes()
            if hashlib.sha256(result_bytes).hexdigest() != expected:
                raise ValueError('result hash mismatch: '+str(result))
            parsed = json.loads(result_bytes, parse_float=Decimal)
            valid = (summary.get('strict_validation', {}).get('valid') is True
                     and summary.get('primal_bound') is not None)
            primal = None
            objective_source = None
            if valid:
                exact = summary['strict_validation'].get('exact', {}).get('objective')
                if exact is not None:
                    primal = Fraction(exact)
                    objective_source = 'GMP original-model objective of serialized point'
                else:
                    primal = number(parsed.get('objective'))
                    objective_source = 'reported objective; exact diagnostic unavailable'
                    warnings.append(dict(run=str(run), job=summary['job_id'],
                                         error='exact_objective_unavailable_using_reported_value'))
            group['records'].append(dict(source=str(result.resolve()),
                                         solver=summary['solver'],
                                         status=summary.get('solver_status'),
                                         primal=primal, objective_source=objective_source,
                                         reported_objective_token=str(parsed.get('objective')),
                                         objective_is_exact=(objective_source == 'GMP original-model objective of serialized point'),
                                         dual=number(summary.get('dual_bound'))))
    return groups, sources, warnings


def diagnose(groups, tolerance):
    findings, comparisons, references = [], 0, []
    for (name, input_sha), group in groups.items():
        sign = -1 if group['maximize'] else 1
        valid = [r for r in group['records'] if r['primal'] is not None]
        if not valid:
            continue
        best = min(valid, key=lambda r: sign*r['primal'])
        references.append(dict(name=name, input_sha256=input_sha, maximize=group['maximize'],
                               point_source=best['source'], point_solver=best['solver'],
                               objective_used=str(best['primal']),
                               objective_exact=str(best['primal']) if best['objective_is_exact'] else None,
                               objective_source=best['objective_source'],
                               reported_objective_token=best['reported_objective_token']))
        for record in group['records']:
            if record['dual'] is not None:
                comparisons += 1
                excess = sign*(record['dual']-best['primal'])
                if excess > tolerance:
                    findings.append(dict(type='reported_dual_exceeds_verified_point',
                                         name=name, input_sha256=input_sha,
                                         maximize=group['maximize'],
                                         bound_source=record['source'],
                                         bound_solver=record['solver'],
                                         reported_dual=str(record['dual']),
                                         point_source=best['source'],
                                         point_solver=best['solver'],
                                         point_objective_used=str(best['primal']),
                                         point_objective_exact=str(best['primal']) if best['objective_is_exact'] else None,
                                         point_objective_source=best['objective_source'],
                                         absolute_excess=str(excess)))
            if str(record['status']).lower() == 'infeasible':
                findings.append(dict(type='infeasibility_claim_with_tolerance_checked_point',
                                     name=name, input_sha256=input_sha,
                                     claim_source=record['source'], point_source=best['source']))
    return findings, comparisons, references


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('runs', nargs='+', type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    if len({p.resolve() for p in args.runs}) != len(args.runs):
        parser.error('duplicate run directories')
    tolerance = Fraction(1, 1000000)
    groups, sources, warnings = collect(args.runs)
    findings, comparisons, references = diagnose(groups, tolerance)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    atomic_json(args.output, dict(updated_utc=utc(),
                                 status='diagnostic_not_a_dual_certificate',
                                 absolute_tolerance=str(tolerance),
                                 input_groups=len(groups), bound_comparisons=comparisons,
                                 findings=findings, reference_points=references, warnings=warnings, sources=sources,
                                 limitation='Points passed absolute-tolerance checks, not necessarily zero-tolerance feasibility. Findings require review and can reflect allowed point tolerances or numerical reporting; absence of findings does not certify bounds or replay optimality proofs.'))
    print(json.dumps(dict(groups=len(groups), comparisons=comparisons,
                          findings=len(findings), warnings=len(warnings))))
    return 2 if findings else 0


if __name__ == '__main__':
    raise SystemExit(main())
