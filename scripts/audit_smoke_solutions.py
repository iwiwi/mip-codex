#!/usr/bin/env python3
"""開発smokeの保存解を元MPSに再照合。元runは変更しない。"""
import argparse
import json
from pathlib import Path
import subprocess
import sys
from benchmark_common import atomic_json, sha256, utc
from verify_cover_cuts import validate_cut_proofs

root = Path(__file__).resolve().parents[1]
parser = argparse.ArgumentParser()
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('runs', nargs='+', type=Path)
args = parser.parse_args()
manifest = {r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
checker = root/'third_party/miplib-checker/bin/solchecker'
for run in args.runs:
    if not (run/'experiment.json').is_file():
        raise ValueError('smoke experiment not found: '+str(run))
args.output.mkdir(parents=True, exist_ok=True)
records = []
cut_records = []
for run in args.runs:
    for result in sorted(run.glob('*/*/result.json')):
        value = json.loads(result.read_text())
        name = result.parent.parent.name
        original = manifest[name]
        assert original['split'] == 'development'
        model = root/original['path']
        assert sha256(model) == original['sha256']
        destination = args.output/run.name/name/result.parent.name
        destination.mkdir(parents=True, exist_ok=True)
        if 'cut_proofs' in value:
            cut_check=validate_cut_proofs(model,value)
            atomic_json(destination/'cut-validation.json',cut_check)
            cut_records.append(dict(run=run.name,name=name,solver=result.parent.name,solution_sha256=sha256(result),validation=cut_check))
        if value.get('values') is None:
            continue

        command = [sys.executable, str(root/'scripts/benchmark_validate.py'), str(model), str(result.resolve()), str(checker), str(destination)]
        with (destination/'audit.log').open('w') as stream:
            completed = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT)
        checks = {}
        for key in ('strict', 'official'):
            path = destination/(key+'-validation.json')
            checks[key] = json.loads(path.read_text()) if path.is_file() else dict(valid=None, error='validation output missing')
        records.append(dict(run=run.name, name=name, solver=result.parent.name,
                            solution_sha256=sha256(result), input_sha256=original['sha256'],
                            exit_code=completed.returncode, command=command, **checks))
summary = dict(updated_utc=utc(), checked=len(records),
               cut_runs=len(cut_records),cut_count=sum(r['validation']['checked'] for r in cut_records),
               cut_valid=sum(r['validation']['valid'] is True for r in cut_records),
               cut_invalid=sum(r['validation']['valid'] is False for r in cut_records),
               cut_unverified=sum(r['validation']['valid'] is None for r in cut_records),
               strict_valid=sum(r['strict']['valid'] is True for r in records),
               strict_invalid=sum(r['strict']['valid'] is False for r in records),
               strict_unverified=sum(r['strict']['valid'] is None for r in records),
               official_valid=sum(r['official']['valid'] is True for r in records))
atomic_json(args.output/'records.json', records)
atomic_json(args.output/'cut-records.json', cut_records)
atomic_json(args.output/'summary.json', summary)
print(json.dumps(summary))
sys.exit(0 if summary['checked'] == summary['strict_valid'] == summary['official_valid'] and summary['cut_runs']==summary['cut_valid'] else 1)
