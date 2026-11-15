#!/usr/bin/env python3
"""桁落ち回帰を元MPSのGMP checkerへ直接照合する。最適化は自作Rustだけ。"""
import json,os,re,subprocess
from pathlib import Path
from benchmark_common import sha256,require_solver_origin
from validate_solution import absolute_checker
root=Path(__file__).resolve().parents[1];binary=root/'target/release/mip-codex';checker=absolute_checker()
assert checker.is_file();info=require_solver_origin(root,binary)
out=root/os.environ.get('MIP_TRANSFORMATION_DIR','artifacts/exact-transformation-gmp');out.mkdir(exist_ok=True)
meta=dict(build_info=info,binary_sha256=sha256(binary),checker_sha256=sha256(checker),git_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip())
(out/'experiment.json').write_text(json.dumps(meta,indent=2)+'\n')
records=[]
for model in sorted((root/'tests/fixtures/exact-transformation').glob('*.mps')):
 for engine in ['dense','revised']:
    directory=out/model.stem/engine;directory.mkdir(parents=True,exist_ok=True)
    env=dict(os.environ,MIP_LP_ENGINE=engine,MIP_OBJECTIVE_ROUNDING='0',MIP_PRIMAL_REPAIR='0',MIP_LP_DIVING='0',MIP_PRIMAL_PROBE='0',MIP_RINS='0',MIP_COVER_CUTS='0',MIP_STRONG_BRANCHING='0',MIP_PROPAGATION='0')
    command=[str(binary),'solve',str(model),'5'];(directory/'command.json').write_text(json.dumps(command)+'\n')
    with (directory/'stdout.log').open('w') as stream,(directory/'stderr.log').open('w') as err:completed=subprocess.run(command,env=env,stdout=stream,stderr=err,timeout=15)
    r=json.loads((directory/'stdout.log').read_text());(directory/'result.json').write_text(json.dumps(r,indent=2)+'\n')
    try:
        assert completed.returncode==0,completed.returncode
        if model.stem in ('unrepresentable-shift','product-overflow','decimal-objective','decimal-row','serialized-point','rounded-coefficient','rounded-fixed-bound'):
            assert r['status']=='numerical_error' and r['values']is None
            record=dict(model=model.name,engine=engine,status='unresolved',reason='original decimal or absolute transformation precision unavailable')
        else:
            assert r['status']=='optimal' and r['objective']==1.,r
            solution=directory/'point.sol';solution.write_text('=obj= '+str(r['objective'])+'\n'+''.join(f'{name} {value}\n'for name,value in r['values'].items()))
            command=[str(checker),str(model),str(solution),'1e-6','1e-6'];(directory/'checker-command.json').write_text(json.dumps(command)+'\n')
            check=subprocess.run(command,capture_output=True,text=True,timeout=30);text=check.stdout+'\n'+check.stderr;(directory/'checker.log').write_text(text)
            assert check.returncode==0 and 'Read MPS: 1'in text and 'Read SOL: 1'in text and 'Check SOL: Integrality 1 Constraints 1 Objective 1'in text,text
            record=dict(model=model.name,engine=engine,status='valid_optimal_fixed_or_forced_point',objective=r['objective'],checker_sha256=meta['checker_sha256'])
    except Exception as error:record=dict(model=model.name,engine=engine,status='fail',error=str(error))
    records.append(record)
summary=dict(cases=len(records),valid_points=sum(r['status']=='valid_optimal_fixed_or_forced_point'for r in records),unresolved=sum(r['status']=='unresolved'for r in records),failures=sum(r['status']=='fail'for r in records),records=records)
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary));raise SystemExit(bool(summary['failures']))
