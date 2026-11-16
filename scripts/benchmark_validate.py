#!/usr/bin/env python3
"""元MPSへ独立した2つのreader/数値系で検査。最適化はしない。"""
import json
from decimal import Decimal
from pathlib import Path
import re
import subprocess
import sys
from benchmark_common import atomic_json, sha256
from validate_solution import validate

model, solution, checker, directory = sys.argv[1:]
directory = Path(directory)
value = json.load(open(solution), parse_float=Decimal)
try:
    strict = validate(model, value)
except Exception as error:
    strict = dict(valid=None, error=str(error))
if value.get('parity_proof') is not None:
    from verify_parity import validate_parity_proof
    strict['parity_proof']=validate_parity_proof(Path(model),value)
    if strict['parity_proof']['valid'] is not True:strict['valid']=False
atomic_json(directory/'strict-validation.json', strict)
sol = directory/'solution.sol'
with sol.open('w') as stream:
    stream.write('=obj= '+str(value['objective'])+'\n')
    for name, x in value['values'].items():
        if any(c.isspace() for c in name):
            raise ValueError('official solution format does not support whitespace names')
        stream.write(name+' '+str(x)+'\n')
command = [checker, model, str(sol), '1e-6', '1e-6']
with (directory/'official-checker.log').open('w') as stream:
    completed = subprocess.run(command, stdout=stream, stderr=subprocess.STDOUT)
text = (directory/'official-checker.log').read_text()
match = re.search(r'Check SOL: Integrality ([01]) Constraints ([01]) Objective ([01])', text)
official = dict(valid=all(v == '1' for v in match.groups()) if match and completed.returncode == 0 else None,
                exit_code=completed.returncode, command=command,
                checker_sha256=sha256(checker), linear_tolerance=1e-6, integer_tolerance=1e-6,
                arithmetic='GMP rational; official MIPLIB relative linear/bound scaling')
if match:
    official['integrality'], official['constraints'], official['objective'] = [v == '1' for v in match.groups()]
atomic_json(directory/'official-validation.json', official)
