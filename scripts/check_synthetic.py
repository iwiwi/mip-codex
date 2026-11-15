#!/usr/bin/env python3
"""固定seedの有界小MILP/LPをHiGHSと比較。純整数では全列挙も照合。"""
import hashlib
import itertools
import math
import json
import os
from pathlib import Path
import random
import subprocess
import sys
import tempfile
import highspy
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs

root = Path(__file__).resolve().parents[1]
rng = random.Random(20261115)
count = int(sys.argv[1]) if len(sys.argv)>1 else 500
failed = 0
larger = os.environ.get('MIP_SYNTHETIC_PROFILE') == 'larger'
binary = os.environ.get('MIP_SYNTHETIC_PROFILE') == 'binary'
outdir = root/os.environ.get('MIP_SYNTHETIC_DIR','artifacts/synthetic')
outdir.mkdir(exist_ok=True)
for case in range(count):
    n=rng.randint(5,10) if binary else rng.randint(8,12) if larger else rng.randint(1,5)
    m=rng.randint(5,12) if binary else rng.randint(8,16) if larger else rng.randint(0,7)
    integer=[binary or case%3==0 or (case%3==1 and rng.random()<0.5) for _ in range(n)]
    bounds=[(0,1) if binary else (rng.randint(-3,0),rng.randint(1,4)) for _ in range(n)]
    cost=[rng.randint(-5,5) for _ in range(n)]
    offset=rng.randint(-4,4)
    maximize=rng.choice([False,True])
    rows=[([rng.randint(-4,4) for _ in range(n)],rng.randint(-10,10),rng.choice(['L','G','E'])) for _ in range(m)]
    if larger or binary:
        planted=[rng.randint(lo,up) for lo,up in bounds]
        rows=[(a,sum(v*x for v,x in zip(a,planted))+rng.randint(0,5)*(1 if kind=='L' else -1 if kind=='G' else 0),kind) for a,_,kind in rows]
    lines=['NAME synthetic','OBJSENSE',' MAX' if maximize else ' MIN','ROWS',' N obj']
    lines += [f' {kind} r{i}' for i,(_,_,kind) in enumerate(rows)]
    lines+=['COLUMNS']
    for j in range(n):
        lines+=[f' x{j} obj {cost[j]}']
        lines += [f' x{j} r{i} {a[j]}' for i,(a,_,_) in enumerate(rows) if a[j]]
    lines+=['RHS',f' rhs obj {-offset}']
    lines += [f' rhs r{i} {b}' for i,(_,b,_) in enumerate(rows)]
    lines+=['BOUNDS']
    for j,(lo,up) in enumerate(bounds):
        lines += [f' {"LI" if integer[j] else "LO"} bounds x{j} {lo}', f' UP bounds x{j} {up}']
    lines+=['ENDATA']
    text='\n'.join(lines)+'\n'
    try:
        with tempfile.TemporaryDirectory() as directory:
            path=Path(directory)/'case.mps';path.write_text(text)
            result=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'solve',str(path),'10'],timeout=15))
            if 'MIP_LP_DIVING' in os.environ:
                assert result.get('lp_diving') == (os.environ['MIP_LP_DIVING'] == '1'), 'unexpected diving mode'
            cut_check=validate_cut_proofs(path,result)
            assert cut_check['valid'] is True, cut_check
            if 'MIP_COVER_CUTS' in os.environ:
                assert result.get('cover_cuts_enabled') == (os.environ['MIP_COVER_CUTS']=='1'), 'unexpected cover mode'
            if 'MIP_PRIMAL_PROBE' in os.environ:
                assert result.get('primal_probe') == (os.environ['MIP_PRIMAL_PROBE']=='1'), 'unexpected probe mode'
            if 'MIP_SEARCH' in os.environ:
                assert result.get('search_strategy') == {'depth':'DepthFirst','best':'BestBound','hybrid':'Hybrid'}[os.environ['MIP_SEARCH']], 'unexpected search strategy'
            if 'MIP_LP_ENGINE' in os.environ:
                assert result.get('lp_engine')=={'dense':'Dense','revised':'Revised'}[os.environ['MIP_LP_ENGINE']], 'unexpected LP engine'
            if 'MIP_BRANCHING' in os.environ:
                assert result.get('branching') == {'fractional':'Fractional','pseudocost':'Pseudocost'}[os.environ['MIP_BRANCHING']], 'unexpected branching mode'
            if 'MIP_CHILD_ORDER' in os.environ:
                assert result.get('child_order') == {'down':'Down','nearest':'Nearest','estimate':'Estimate'}[os.environ['MIP_CHILD_ORDER']], 'unexpected child order'
            if 'MIP_PRIMAL_REPAIR' in os.environ:
                assert result.get('primal_repair')==(os.environ['MIP_PRIMAL_REPAIR']=='1'), 'binary does not match requested repair mode'
            if 'MIP_PROPAGATION' in os.environ:
                assert result.get('propagation')==(os.environ['MIP_PROPAGATION']=='1'), 'binary does not match requested propagation mode'
            h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('mip_rel_gap',0.);h.setOptionValue('mip_abs_gap',0.);h.readModel(str(path));h.run()
            status=h.getModelStatus()
            if status==highspy.HighsModelStatus.kInfeasible:
                assert result['status']=='infeasible', result
            else:
                assert status==highspy.HighsModelStatus.kOptimal, str(status)
                assert result['status']=='optimal', result
                expected=h.getObjectiveValue()
                assert abs(expected-result['objective'])<=1e-6*(1+abs(expected)),(expected,result)
                assert validate(path,result)['valid'], result
                if case==0:
                    (outdir/'example.mps').write_text(text)
                    (outdir/'example.solution.json').write_text(json.dumps(result,indent=2)+'\n')
            exhaustive=all(integer) and math.prod(up-lo+1 for lo,up in bounds)<=500000
            if exhaustive:
                feasible=[]
                for x in itertools.product(*(range(lo,up+1) for lo,up in bounds)):
                    def satisfies(row):
                        a,b,kind=row;activity=sum(v*t for v,t in zip(a,x))
                        return activity<=b if kind=='L' else activity>=b if kind=='G' else activity==b
                    if all(map(satisfies,rows)):
                        feasible.append(offset+sum(v*t for v,t in zip(cost,x)))
                if not feasible: assert result['status']=='infeasible'
                else: assert abs((max(feasible) if maximize else min(feasible))-result['objective'])<1e-6
            print(json.dumps(dict(case=case,status='pass',solver_status=result['status'],nodes=result['nodes'],cut_count=len(result.get('cut_proofs',[])),cut_rounds=result.get('cut_rounds'),cut_lp_solves=result.get('cut_lp_solves'),cut_verified=cut_check['valid'],search=result.get('search_strategy'),best_pops=result.get('search_best_pops'),max_open=result.get('max_open_nodes'),lp_solves=result.get('lp_solves'),branching=result.get('branching'),child_order=result.get('child_order'),pseudocost_selections=result.get('pseudocost_selections'),scaled_columns=result.get('lp_scaled_columns'),scaled_rows=result.get('lp_scaled_rows'),primal_repair=result.get('primal_repair'),lp_diving=result.get('lp_diving'),primal_probe=result.get('primal_probe'),probe=result.get('probe'),diving=result.get('diving'),repair_improvements=result.get('repair_improvements'),propagation=result.get('propagation'),propagation_pruned=result.get('propagation_pruned'),exhaustive=exhaustive,warm_successes=result.get('lp_warm_successes'),warm_fallbacks=result.get('lp_warm_fallbacks'),dual_pivots=result.get('lp_dual_pivots'),sha256=hashlib.sha256(text.encode()).hexdigest())),flush=True)
    except Exception as error:
        failed+=1
        (outdir/f'failure-{case}.mps').write_text(text)
        print(json.dumps(dict(case=case,status='fail',error=str(error))),flush=True)
print(json.dumps(dict(cases=count,failures=failed)),flush=True)
sys.exit(bool(failed))
