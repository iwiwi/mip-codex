#!/usr/bin/env python3
"""固定runを変更せず、完了済み保存解を新primaryで再監査する。"""
import argparse
from decimal import Decimal
import json
from pathlib import Path
from benchmark_common import atomic_json, sha256, utc
from validate_solution import validate

parser = argparse.ArgumentParser()
parser.add_argument('run', type=Path)
parser.add_argument('output', type=Path)
args = parser.parse_args()
config = json.loads((args.run/'config.json').read_text())
args.output.mkdir(parents=True, exist_ok=True)
records = []
for job in config['jobs']:
    directory = args.run/'cases'/job['id']
    if not (directory/'summary.json').is_file() or not (directory/'result.json').is_file():
        continue
    result_path = directory/'result.json'
    result = json.loads(result_path.read_text(), parse_float=Decimal)
    if result.get('values') is None:
        continue
    instance = config['instances'][job['instance_index']]
    model = Path(instance['absolute_path'])
    if sha256(model) != instance['sha256']:
        raise ValueError('input hash mismatch')
    try:
        check = validate(model, result)
    except Exception as error:
        check = dict(valid=None, error=str(error))
    record = dict(job_id=job['id'], input_sha256=instance['sha256'],
                  solution_sha256=sha256(result_path), validation=check)
    atomic_json(args.output/(job['id']+'.json'), record)
    records.append(record)
summary = dict(updated_utc=utc(), run=str(args.run.resolve()), checked=len(records),
               valid=sum(r['validation']['valid'] is True for r in records),
               invalid=sum(r['validation']['valid'] is False for r in records),
               unverified=sum(r['validation']['valid'] is None for r in records))
atomic_json(args.output/'summary.json', summary)
print(json.dumps(summary))
