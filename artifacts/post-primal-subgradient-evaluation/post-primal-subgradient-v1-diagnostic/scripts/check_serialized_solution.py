#!/usr/bin/env python3
"""全固定モデルのJSONを元MPSのGMPとFractionで直接検査する。"""
import json,os,subprocess
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
from benchmark_common import require_solver_origin,sha256
from validate_solution import absolute_checker
from verify_cover_cuts import exact_model
root=Path(__file__).resolve().parents[1];binary=root/'target/release/mip-codex';checker=absolute_checker()
assert checker.is_file();info=require_solver_origin(root,binary)
out=root/os.environ.get('MIP_SERIALIZATION_DIR','artifacts/serialized-solution-check');out.mkdir(parents=True,exist_ok=True)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),checker_sha256=sha256(checker)),indent=2)+'\n')
records=[]
for model in sorted((root/'tests/fixtures/serialization').glob('*.mps')):
 names,variables,rows,cost,offset=exact_model(model,True)
 assert not rows and all(v['lower']==v['upper'] and v['lower']is not None for v in variables.values())
 expected=offset+sum((cost.get(n,Fraction(0))*variables[n]['lower']for n in names),Fraction(0))
 for engine in ['dense','revised']:
  directory=out/model.stem/engine;directory.mkdir(parents=True,exist_ok=True)
  env=dict(os.environ,MIP_LP_ENGINE=engine)
  with (directory/'stdout.log').open('w')as stream,(directory/'stderr.log').open('w')as err:completed=subprocess.run([str(binary),'solve',str(model),'5'],env=env,stdout=stream,stderr=err,timeout=15)
  result=json.loads((directory/'stdout.log').read_text(),parse_float=Decimal)
  try:
   assert completed.returncode==0 and result['status']=='optimal',result
   assert Fraction(result['objective'])==expected and Fraction(result['dual_bound'])==expected
   assert set(result['values'])==set(names)
   assert all(Fraction(result['values'][n])==variables[n]['lower']for n in names)
   solution=directory/'point.sol';solution.write_text('=obj= '+str(result['objective'])+'\n'+''.join(f'{n} {result["values"][n]}\n'for n in names))
   command=[str(checker),str(model),str(solution),'1e-6','1e-6'];(directory/'checker-command.json').write_text(json.dumps(command)+'\n')
   check=subprocess.run(command,capture_output=True,text=True,timeout=30);text=check.stdout+'\n'+check.stderr;(directory/'checker.log').write_text(text)
   assert check.returncode==0 and 'Check SOL: Integrality 1 Constraints 1 Objective 1'in text,text
   record=dict(model=model.name,engine=engine,status='valid_optimal_unique_fixed_point')
  except Exception as error:record=dict(model=model.name,engine=engine,status='fail',error=str(error))
  records.append(record)
summary=dict(cases=len(records),failures=sum(r['status']=='fail'for r in records),records=records)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(summary['failures']))
