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
import tempfile
from validate_solution import validate, absolute_checker
from benchmark_common import require_solver_origin
from verify_cover_cuts import validate_cut_proofs

root=Path(__file__).resolve().parents[1]
if not absolute_checker().is_file():
    raise SystemExit('required absolute rational checker unavailable: '+str(absolute_checker()))
solver_build_info=require_solver_origin(root,root/'target/release/mip-codex')
seconds=int(sys.argv[1]) if len(sys.argv)>1 else 10
names=sys.argv[2:] or ['pk1.mps.gz','mad.mps.gz','markshare_4_0.mps.gz']
manifest=json.load(open(root/'data/manifest.json'))
records={r['name']:r for r in manifest['instances']}
output=Path(tempfile.mkdtemp(prefix='smoke-'+time.strftime('%Y%m%d-%H%M%S',time.gmtime())+'-',dir=root/'artifacts'))
core=min(os.sched_getaffinity(0))
env=os.environ.copy();env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
solvers=os.environ.get('MIP_BENCH_SOLVERS','rust,highs,scip,glpk').split(',')
source_files=sorted([*root.glob('src/**/*.rs'),*root.glob('scripts/*.py'),root/'Cargo.toml',root/'Cargo.lock',root/'build.rs',root/'tools/glpk-benchmark/main.c'])
source_hashes={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in source_files}
meta=dict(solvers=solvers,source_sha256=source_hashes,binary_sha256=hashlib.sha256((root/'target/release/mip-codex').read_bytes()).hexdigest(),seconds=seconds,threads=1,cpu=core,memory_bytes=16*1024**3,git_sha=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),dirty=bool(subprocess.check_output(['git','status','--porcelain','--','src','scripts','Cargo.toml','Cargo.lock'],cwd=root)),names=names)
meta['solver_build_info']=solver_build_info
meta['absolute_checker_sha256']=hashlib.sha256(absolute_checker().read_bytes()).hexdigest()
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
            branching_modes = {
                'rust-fractional': ('fractional', 'down'),
                'rust-nearest': ('fractional', 'nearest'),
                'rust-pseudocost': ('pseudocost', 'down'),
                'rust-pseudocost-nearest': ('pseudocost', 'nearest'),
                'rust-pseudocost-estimate': ('pseudocost', 'estimate'),
            }
            queue_modes = {
                'rust-depth': ('depth', 'down'),
                'rust-best': ('best', 'down'),
                'rust-hybrid': ('hybrid', 'down'),
                'rust-best-estimate': ('best', 'estimate'),
                'rust-hybrid-estimate': ('hybrid', 'estimate'),
            }
            strong_modes={'rust-strong-classic':('0','0'),'rust-strong-objective':('1','0'),'rust-strong-hints':('0','1'),'rust-strong-objective-hints':('1','1')}
            if solver in strong_modes:
                objective,hints=strong_modes[solver]
                run_env.update(MIP_STRONG_OBJECTIVE=objective,MIP_STRONG_INFEASIBLE_HINT=hints,MIP_STRONG_BRANCHING='1',MIP_OBJECTIVE_ROUNDING='1',MIP_COVER_ROUNDS='2',MIP_RINS='1',MIP_ROUNDING_CUTS='0',MIP_COVER_CUTS='1',MIP_PRIMAL_PROBE='1',MIP_SEARCH='depth',MIP_BRANCHING='pseudocost',MIP_CHILD_ORDER='down',MIP_LP_ENGINE='revised',MIP_LP_WARM_START='1',MIP_PROPAGATION='0',MIP_PRIMAL_REPAIR='1',MIP_LP_DIVING='1')
            elif solver in ('rust-cuts-two', 'rust-cuts-four'):
                run_env.update(MIP_COVER_ROUNDS='2' if solver=='rust-cuts-two' else '4', MIP_OBJECTIVE_ROUNDING='0', MIP_RINS='1', MIP_STRONG_BRANCHING='1', MIP_ROUNDING_CUTS='0', MIP_COVER_CUTS='1', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-objective', 'rust-no-objective'):
                run_env.update(MIP_OBJECTIVE_ROUNDING='1' if solver=='rust-objective' else '0', MIP_RINS='1', MIP_STRONG_BRANCHING='1', MIP_ROUNDING_CUTS='0', MIP_COVER_CUTS='1', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-rins', 'rust-no-rins'):
                run_env.update(MIP_RINS='1' if solver=='rust-rins' else '0', MIP_STRONG_BRANCHING='1', MIP_ROUNDING_CUTS='0', MIP_COVER_CUTS='1', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-strong', 'rust-no-strong'):
                run_env.update(MIP_STRONG_BRANCHING='1' if solver=='rust-strong' else '0', MIP_ROUNDING_CUTS='0', MIP_COVER_CUTS='1', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-rounding', 'rust-no-rounding'):
                run_env.update(MIP_ROUNDING_CUTS='1' if solver=='rust-rounding' else '0', MIP_COVER_CUTS='1', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in queue_modes:
                search, order = queue_modes[solver]
                run_env.update(MIP_SEARCH=search, MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER=order, MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in branching_modes:
                rule, order = branching_modes[solver]
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_BRANCHING=rule, MIP_CHILD_ORDER=order, MIP_SEARCH='depth')
            elif solver in ('rust-cuts', 'rust-no-cuts'):
                run_env.update(MIP_COVER_CUTS='1' if solver=='rust-cuts' else '0', MIP_PRIMAL_PROBE='1', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-probe', 'rust-no-probe'):
                run_env.update(MIP_PRIMAL_PROBE='1' if solver=='rust-probe' else '0', MIP_SEARCH='depth', MIP_BRANCHING='pseudocost', MIP_CHILD_ORDER='down', MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1')
            elif solver in ('rust-dive', 'rust-no-dive'):
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1', MIP_LP_DIVING='1' if solver=='rust-dive' else '0')
            elif solver in ('rust-repair', 'rust-no-repair'):
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='0', MIP_PRIMAL_REPAIR='1' if solver=='rust-repair' else '0')
            elif solver in ('rust-prop', 'rust-no-prop'):
                run_env.update(MIP_LP_ENGINE='revised', MIP_LP_WARM_START='1', MIP_PROPAGATION='1' if solver=='rust-prop' else '0')
            elif solver in ('rust-cold','rust-warm'):
                run_env['MIP_LP_ENGINE']='revised'
                run_env['MIP_LP_WARM_START']='1' if solver=='rust-warm' else '0'
            elif solver.startswith('rust-'):run_env['MIP_LP_ENGINE']=solver.removeprefix('rust-')
            (directory/'environment.json').write_text(json.dumps({k:run_env.get(k) for k in ['MIP_PARITY_EQUALITIES','MIP_DEFER_INFEASIBILITY','MIP_GRAPH_ADAPTIVE_LOCAL','MIP_GRAPH_LOCAL_DIVERSIFY','MIP_GRAPH_ROOT_PREFIX','MIP_GRAPH_ROOT_COLORING','MIP_GRAPH_LOCAL_SEARCH','MIP_BINARY_GRAPH','MIP_BOX_PRIMAL','MIP_LP_WORKSPACE_MIB','MIP_FIXED_COLUMNS','OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS','MIP_LP_ENGINE','MIP_LP_WARM_START','MIP_PROPAGATION','MIP_PRIMAL_REPAIR','MIP_BRANCHING','MIP_CHILD_ORDER','MIP_LP_DIVING','MIP_SEARCH','MIP_PRIMAL_PROBE','MIP_MIR_CUTS','MIP_CLIQUE_CUTS','MIP_AGGREGATE_CUTS','MIP_COVER_CUTS','MIP_ROUNDING_CUTS','MIP_STRONG_BRANCHING','MIP_RINS','MIP_OBJECTIVE_ROUNDING','MIP_COVER_ROUNDS','MIP_STRONG_OBJECTIVE','MIP_STRONG_INFEASIBLE_HINT']})+'\n')
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
                if solver.startswith('rust') and run_env.get('MIP_LP_WORKSPACE_MIB') is not None:
                    assert parsed['lp_workspace_bytes'] == int(run_env['MIP_LP_WORKSPACE_MIB'])*1024**2
                if solver.startswith('rust') and run_env.get('MIP_BOX_PRIMAL') is not None:
                    assert parsed['box_primal_enabled'] == (run_env['MIP_BOX_PRIMAL']=='1')
                if solver.startswith('rust') and run_env.get('MIP_BINARY_GRAPH') is not None:
                    assert parsed['binary_graph_enabled'] == (run_env['MIP_BINARY_GRAPH']=='1')
                if solver in ('rust-cuts', 'rust-no-cuts'):
                    assert parsed['cover_cuts_enabled'] == (solver == 'rust-cuts')
                if solver in ('rust-probe', 'rust-no-probe'):
                    assert parsed['primal_probe'] == (solver == 'rust-probe')
                    assert parsed['search_strategy'] == 'DepthFirst' and parsed['child_order'] == 'Down'
                if solver in ('rust-cuts-two', 'rust-cuts-four'):
                    assert parsed['cover_round_limit'] == (2 if solver=='rust-cuts-two' else 4)
                if solver in ('rust-objective', 'rust-no-objective'):
                    assert parsed['objective_rounding'] == (solver == 'rust-objective')
                if solver in strong_modes:
                    assert parsed['strong_objective']==(strong_modes[solver][0]=='1') and parsed['strong_infeasible_hint']==(strong_modes[solver][1]=='1')
                if solver in ('rust-rins', 'rust-no-rins'):
                    assert parsed['rins_enabled'] == (solver == 'rust-rins')
                if solver in ('rust-strong', 'rust-no-strong'):
                    assert parsed['strong_branching'] == (solver == 'rust-strong')
                if solver in ('rust-rounding', 'rust-no-rounding'):
                    assert parsed['rounding_cuts_enabled'] == (solver == 'rust-rounding')
                if solver in queue_modes:
                    search, order = queue_modes[solver]
                    assert parsed['search_strategy'] == {'depth':'DepthFirst','best':'BestBound','hybrid':'Hybrid'}[search]
                    assert parsed['child_order'] == {'down':'Down','estimate':'Estimate'}[order]
                    assert parsed['lp_diving'] and parsed['primal_repair'] and not parsed['propagation']
                if solver in branching_modes:
                    rule, order = branching_modes[solver]
                    assert parsed['branching'] == {'fractional':'Fractional','pseudocost':'Pseudocost'}[rule]
                    assert parsed['child_order'] == {'down':'Down','nearest':'Nearest','estimate':'Estimate'}[order]
                if solver in ('rust-dive', 'rust-no-dive'):
                    assert parsed['lp_diving'] == (solver == 'rust-dive'), 'unexpected diving mode'
                if solver in ('rust-repair', 'rust-no-repair'):
                    assert parsed['primal_repair'] == (solver == 'rust-repair'), 'unexpected repair mode'
                if solver in ('rust-prop', 'rust-no-prop'):
                    assert parsed['propagation'] == (solver == 'rust-prop'), 'unexpected propagation mode'
                result.update({k:v for k,v in parsed.items() if k!='values'})
                if 'cut_proofs' in parsed or parsed.get('objective_grid') is not None:
                    cut_check=validate_cut_proofs(path,parsed)
                    (directory/'cut-validation.json').write_text(json.dumps(cut_check)+'\n')
                    result['cut_validation']=cut_check
                    if cut_check['valid'] is not True:
                        result['status']='invalid_cut_proof' if cut_check['valid'] is False else 'unverified_cut_proof'
                        result['native_dual_bound']=result.get('dual_bound');result['dual_bound']=None;result['gap']=None

                if solver.startswith('rust'):(directory/'result.json').write_text(json.dumps(parsed)+'\n')
                if parsed.get('values') is not None:
                    validation=validate(path,parsed)
                    (directory/'validation.json').write_text(json.dumps(validation)+'\n')
                    result['validation']=validation
            except Exception as error:result.update(status='error',error=str(error))
            summary.write(json.dumps(result,allow_nan=False)+'\n');summary.flush()
            print(name,solver,result.get('status'),flush=True)
print(output)
