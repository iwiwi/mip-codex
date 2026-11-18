#!/usr/bin/env python3
"""Read-only final scoring. Does not optimize or read model files."""
import argparse
import collections
from fractions import Fraction
import hashlib
import json
import math
from pathlib import Path


def digest(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def number(value):
    if value is None or isinstance(value, bool):
        return None
    try:
        return Fraction(str(value))
    except (ValueError, ZeroDivisionError):
        return None


def classify(row, limit, maximize=False):
    strict = row.get('strict_validation') or {}
    primal = None
    if strict.get('valid') is True:
        primal = number((strict.get('exact') or {}).get('objective'))
        if primal is None:
            primal = number(row.get('primal_bound'))
    dual = number(row.get('dual_bound'))
    execution = row.get('execution') or {}
    execution_failed = execution.get('exit_code') != 0 or execution.get('watchdog', False) or execution.get('boot_changed', False)
    if execution_failed or row.get('status') in ['watchdog', 'process_error', 'error', 'oom', 'supervisor_crash']:
        primal = dual = None
    cuts = row.get('cut_validation')
    if cuts is not None and cuts.get('valid') is not True:
        dual = None
    signed_inconsistency = False
    if primal is not None and dual is not None:
        tolerance = Fraction(1, 10**8) * (1 + abs(primal))
        signed_inconsistency = (primal-dual if maximize else dual-primal) > tolerance
        if signed_inconsistency:
            dual = None
    gap = abs(primal-dual)/max(Fraction(1), abs(primal)) if primal is not None and dual is not None else None
    clean = (row.get('status') == 'optimal' and execution.get('exit_code') == 0
             and not execution.get('watchdog', False) and not execution.get('boot_changed', False)
             and (cuts is None or cuts.get('valid') is True))
    solved = clean and primal is not None and gap is not None and gap <= Fraction(1, 10**7)
    seconds = number(row.get('seconds'))
    charged = min(float(seconds), limit) if solved and seconds is not None and seconds >= 0 else limit
    return dict(status=row.get('status', 'missing'), solver_status=row.get('solver_status'),
                primal=None if primal is None else str(primal), dual=None if dual is None else str(dual),
                common_gap=None if gap is None else float(gap), signed_bound_inconsistency=signed_inconsistency,
                declared_optimal_with_valid_primal=solved,
                mathematical_optimality_certificate_independently_verified=False,
                strict_valid=strict.get('valid'), official_valid=(row.get('official_validation') or {}).get('valid'),
                solver_seconds=row.get('seconds'), charged_seconds=charged,
                execution_wall_seconds=execution.get('wall_seconds'), result_sha256=row.get('result_sha256'))


def load_run(root):
    root = Path(root)
    config = json.loads((root/'config.json').read_text())
    rows = {}
    sources = []
    for job in config['jobs']:
        instance = config['instances'][job['instance_index']]
        key = (instance['name'], job['solver'])
        if key in rows:
            raise ValueError('duplicate configured case '+repr(key))
        path = root/'cases'/job['id']/'summary.json'
        if path.exists():
            row = json.loads(path.read_text())
            for field, expected in [('name', key[0]), ('solver', key[1]), ('input_sha256', instance['sha256'])]:
                if row.get(field) != expected:
                    raise ValueError('summary identity mismatch '+str(path)+' '+field)
            if row.get('result_sha256'):
                if digest(path.parent/'result.json') != row['result_sha256']:
                    raise ValueError('result hash mismatch '+str(path))
            sources.append(dict(path=str(path.resolve()), sha256=digest(path)))
        else:
            row = dict(name=key[0], solver=key[1], input_sha256=instance['sha256'], status='missing')
        row = dict(row, report_source=str(root.resolve()), report_job_id=job['id'], report_cpu=job['cpu'])
        rows[key] = row
    return config, rows, dict(root=str(root.resolve()), config_sha256=digest(root/'config.json'), summaries=sources)


def combine(runs, replacements=()):
    merged, provenance, originals = {}, [], []
    first = None
    replaced_keys = set()
    input_hashes = {}
    for root in [*runs, *replacements]:
        config, rows, source = load_run(root)
        resources = {k: config[k] for k in ['seconds', 'memory_bytes', 'threads', 'cores', 'read_write_grace_seconds', 'validation_seconds', 'manifest_sha256']}
        if first is None:
            first = resources
        elif resources != first:
            raise ValueError('incompatible evaluation resource/protocol configuration')
        replacing = root in replacements
        for key, row in rows.items():
            if key[0] in input_hashes and input_hashes[key[0]] != row["input_sha256"]:
                raise ValueError("input hash differs across solvers "+key[0])
            input_hashes[key[0]] = row["input_sha256"]
            if replacing:
                if key in replaced_keys:
                    raise ValueError("multiple replacements for one case "+repr(key))
                replaced_keys.add(key)
                if key not in merged or merged[key]['input_sha256'] != row['input_sha256'] or merged[key]['report_cpu'] != row['report_cpu']:
                    raise ValueError('replacement identity/input/CPU mismatch '+repr(key))
                originals.append(merged[key])
            elif key in merged:
                raise ValueError('duplicate case without explicit replacement '+repr(key))
            merged[key] = row
        provenance.append(dict(source, replacement=replacing))
    if first is None:
        raise ValueError('no runs')
    return first, merged, provenance, originals


def pairwise(cases, maximize=False):
    by_solver = collections.defaultdict(dict)
    for case in cases:
        by_solver[case['solver']][case['name']] = case
    comparisons = []
    solvers = sorted(by_solver)
    for i, left in enumerate(solvers):
        for right in solvers[i+1:]:
            shared = sorted(set(by_solver[left]) & set(by_solver[right]))
            counts = {}
            for field in ['primal', 'dual']:
                values = collections.Counter()
                for name in shared:
                    a, b = (number(by_solver[s][name][field]) for s in [left, right])
                    if a is None or b is None:
                        values['neither' if a is None and b is None else 'left_only' if a is not None else 'right_only'] += 1
                    elif abs(a-b) <= Fraction(1,10**8)*(1+max(abs(a),abs(b))):
                        values['equal'] += 1
                    else:
                        larger_better = maximize if field == 'primal' else not maximize
                        values['left_better' if (a>b)==larger_better else 'right_better'] += 1
                counts[field] = dict(values)
            comparisons.append(dict(left=left, right=right, shared_cases=len(shared), **counts))
    return comparisons


def report(resources, rows, provenance, originals, maximize=False):
    limit = resources['seconds']
    cases = []
    groups = collections.defaultdict(list)
    for (name, solver), row in sorted(rows.items()):
        case = dict(name=name, solver=solver, split=row.get('split'), input_sha256=row['input_sha256'],
                    source=row['report_source'], job_id=row['report_job_id'], cpu=row['report_cpu'],
                    **classify(row, limit, maximize))
        cases.append(case)
        groups[solver].append(case)
    summary = {}
    for solver, values in groups.items():
        summary[solver] = dict(cases=len(values), statuses=dict(collections.Counter(v['status'] for v in values)),
            declared_optimal_with_valid_primal=sum(v['declared_optimal_with_valid_primal'] for v in values),
            strict_feasible=sum(v['primal'] is not None for v in values), finite_usable_dual=sum(v['dual'] is not None for v in values),
            signed_bound_inconsistencies=sum(v['signed_bound_inconsistency'] for v in values),
            solver_reported_seconds_over_limit=sum(number(v['solver_seconds']) is not None and number(v['solver_seconds'])>limit for v in values),
            execution_wall_seconds_over_limit_and_grace=sum(number(v['execution_wall_seconds']) is not None and number(v['execution_wall_seconds'])>limit+resources['read_write_grace_seconds'] for v in values),
            shifted_geometric_seconds_10=math.exp(sum(math.log(v['charged_seconds']+10) for v in values)/len(values))-10)
    return dict(resources=resources, maximize=maximize, complete=all(c['status']!='missing' for c in cases),
        optimality_label='solver optimal declaration + independently checked primal + numerical gap; not a rational proof of all search',
        timing='solver-reported seconds, capped at limit; every non-optimal or missing case charged the full limit; shift10',
        summary=summary, cases=cases, pairwise=pairwise(cases,maximize),
        pairwise_tolerance="absolute difference <= 1e-8*(1+max(abs(a),abs(b))); absent values counted separately",
        provenance=provenance, replaced_originals=originals)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('runs', nargs='+', type=Path)
    parser.add_argument('--replacement', action='append', default=[], type=Path)
    parser.add_argument('--maximize', action='store_true')
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    resources, rows, sources, originals = combine(args.runs, args.replacement)
    result = report(resources, rows, sources, originals, args.maximize)
    with args.output.open('x') as stream:
        json.dump(result, stream, ensure_ascii=False, allow_nan=False, indent=2)
        stream.write('\n')
    print(json.dumps(result['summary'], ensure_ascii=False))


if __name__ == '__main__':
    main()
