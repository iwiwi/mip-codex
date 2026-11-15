#!/usr/bin/env python3
"""同じCPU/16 GiB制限での短期比較。全結果・生ログ・解検証を保存。"""
import hashlib
import json
import os
from pathlib import Path
import resource
import subprocess
import sys
import time
from validate_solution import validate

root=Path(__file__).resolve().parents[1]
seconds=int(sys.argv[1]) if len(sys.argv)>1 else 10
names=sys.argv[2:] or ['pk1.mps.gz','mad.mps.gz','markshare_4_0.mps.gz']
manifest=json.load(open(root/'data/manifest.json'))
records={r['name']:r for r in manifest['instances']}
output=root/'artifacts'/('smoke-'+time.strftime('%Y%m%d-%H%M%S',time.gmtime()))
output.mkdir()
core=min(os.sched_getaffinity(0))
env=os.environ.copy();env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
solvers=os.environ.get('MIP_BENCH_SOLVERS','rust,highs,scip,glpk').split(',')
source_files=sorted([*root.glob('src/*.rs'),*root.glob('scripts/*.py'),root/'Cargo.toml',root/'Cargo.lock'])
source_hashes={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files}
meta=dict(solvers=solvers,source_sha256=source_hashes,binary_sha256=hashlib.sha256((root/'target/release/mip-codex').read_bytes()).hexdigest(),seconds=seconds,threads=1,cpu=core,memory_bytes=16*1024**3,git_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain','--','src','scripts','Cargo.toml','Cargo.lock'],cwd=root)),names=names)
(output/'experiment.json').write_text(json.dumps(meta,indent=2)+'\n')

def restrict():
    os.sched_setaffinity(0,{core})
    resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))

with (output/'results.jsonl').open('w') as summary:
    for name in names:
        record=records[name]
        assert record['split']=='development', 'smoke benchmark only uses development split'
        path=root/record['path']
        assert hashlib.sha256(path.read_bytes()).hexdigest()==record['sha256']
        for solver in solvers:
            directory=output/name/solver;directory.mkdir(parents=True)
            run_env=env.copy()
            if solver in ('rust-repair', 'rust-no-repair'):
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1' if solver=='rust-repair' else '0')
            elif solver in ('rust-prop', 'rust-no-prop'):
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='1' if solver=='rust-prop' else '0')
            elif solver in ('rust-cold','rust-warm'):
                run_env['MIP_LP_ENGINE']='revised'
                run_env['MIP_LP_WARM_START','MIP_PROPAGATION','MIP_PRIMAL_REPAIR']='1' if solver=='rust-warm' else '0'
            elif solver.startswith('rust-'):run_env['MIP_LP_ENGINE']=solver.removeprefix('rust-')
            (directory/'environment.json').write_text(json.dumps({k:run_env.get(k) for k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS','MIP_LP_ENGINE','MIP_LP_WARM_START','MIP_PROPAGATION','MIP_PRIMAL_REPAIR']})+'\n')
            if solver.startswith('rust'):
                assert hashlib.sha256((root/'target/release/mip-codex').read_bytes()).hexdigest() == meta['binary_sha256'], 'solver binary changed during smoke run'
            if solver.startswith('rust'):cmd=[str(root/'target/release/mip-codex'),'solve',str(path),str(seconds)]
            else:cmd=[str(root/'.venv/bin/python'),str(root/'scripts/comparison_worker.py'),solver,str(path),str(seconds),str(directory)]
            (directory/'command.json').write_text(json.dumps(cmd)+'\n')
            full=['/usr/bin/time','-v','-o',str(directory/'resource.log'),*cmd]
            started=time.monotonic()
            with (directory/'stdout.log').open('w') as stdout,(directory/'stderr.log').open('w') as stderr:
                p=subprocess.Popen(full,stdout=stdout,stderr=stderr,env=run_env,preexec_fn=restrict,start_new_session=True)
                (directory/'pid').write_text(str(p.pid)+'\n')
                try:exit_code=p.wait(timeout=seconds+60)
                except subprocess.TimeoutExpired:
                    import signal
                    os.killpg(p.pid,signal.SIGKILL);exit_code=p.wait()
            if solver.startswith('rust'):
                assert hashlib.sha256((root/'target/release/mip-codex').read_bytes()).hexdigest() == meta['binary_sha256'], 'solver binary changed during smoke run'
            elapsed=time.monotonic()-started
            (directory/'exit.json').write_text(json.dumps(dict(exit_code=exit_code,wall_seconds=elapsed))+'\n')
            result=dict(name=name,solver=solver,exit_code=exit_code,wall_seconds=elapsed,input_sha256=record['sha256'])
            try:
                parsed=json.loads((directory/('stdout.log' if solver.startswith('rust') else 'result.json')).read_text())
                if solver in ('rust-repair', 'rust-no-repair'):
                    assert parsed['primal_repair'] == (solver == 'rust-repair'), 'unexpected repair mode'
                if solver in ('rust-prop', 'rust-no-prop'):
                    assert parsed['propagation'] == (solver == 'rust-prop'), 'unexpected propagation mode'
                result.update({k:v for k,v in parsed.items() if k!='values'})
                if solver.startswith('rust'):(directory/'result.json').write_text(json.dumps(parsed)+'\n')
                if parsed.get('values') is not None:
                    validation=validate(path,parsed)
                    (directory/'validation.json').write_text(json.dumps(validation)+'\n')
                    result['validation']=validation
            except Exception as error:result.update(status='error',error=str(error))
            summary.write(json.dumps(result,allow_nan=False)+'\n');summary.flush()
            print(name,solver,result.get('status'),flush=True)
print(output)
