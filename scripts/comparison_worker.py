#!/usr/bin/env python3
"""比較専用プロセス。Rustソルバーから呼び出さない。"""
import json
import math
from pathlib import Path
import re
import subprocess
import sys

solver, filename, seconds, directory=sys.argv[1:]
seconds=float(seconds)
out=Path(directory)
def finite(v):
    return float(v) if math.isfinite(v) and abs(v)<1e19 else None

if solver=='highs':
    import highspy
    h=highspy.Highs()
    options=dict(time_limit=seconds,threads=1,random_seed=0,mip_rel_gap=0.,mip_abs_gap=0.,small_matrix_value=1e-12)
    for k,v in options.items():
        assert h.setOptionValue(k,v)==highspy.HighsStatus.kOk, k
    h.writeOptions(str(out/'options.txt'))
    assert h.readModel(filename)!=highspy.HighsStatus.kError
    h.run()
    info=h.getInfo();solution=h.getSolution();lp=h.getLp()
    status=h.modelStatusToString(h.getModelStatus())
    valid=solution.value_valid and info.primal_solution_status==highspy.SolutionStatus.kSolutionStatusFeasible
    result=dict(status=status,objective=finite(info.objective_function_value) if valid else None,
                dual_bound=finite(info.mip_dual_bound),gap=finite(info.mip_gap),nodes=info.mip_node_count,
                seconds=h.getRunTime(),version=h.version(),options=options,
                values=dict(zip(lp.col_names_,solution.col_value)) if valid else None)
elif solver=='scip':
    import pyscipopt
    h=pyscipopt.Model()
    options={'limits/time':seconds,'limits/memory':16384.,'parallel/maxnthreads':1,'lp/threads':1,'randomization/randomseedshift':0,'limits/gap':0.,'limits/absgap':0.}
    for k,v in options.items():h.setParam(k,v)
    h.writeParams(str(out/'options.txt'),comments=True,onlychanged=False)
    h.readProblem(filename);h.optimize()
    solution=h.getBestSol()
    result=dict(status=str(h.getStatus()),objective=finite(h.getObjVal()) if solution else None,
                dual_bound=finite(h.getDualbound()),gap=finite(h.getGap()),nodes=h.getNNodes(),
                seconds=h.getSolvingTime(),version=f'{h.getMajorVersion()}.{h.getMinorVersion()}.{h.getTechVersion()}',options=options,
                values={v.name:h.getSolVal(solution,v) for v in h.getVars(transformed=False)} if solution else None)
elif solver=='glpk':
    cmd=['glpsol','--freemps',filename,'--tmlim',str(int(seconds)),'--memlim','16384','--mipgap','0','--write',str(out/'solution.raw'),'--wglp',str(out/'model.glp')]
    completed=subprocess.run(cmd,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,text=True)
    print(completed.stdout)
    names={}
    if (out/'model.glp').exists():
        for line in (out/'model.glp').read_text().splitlines():
            t=line.split()
            if t[:2]==['n','j']:names[int(t[2])]=t[3]
    result=dict(status='error',objective=None,dual_bound=None,gap=None,nodes=None,version='5.0',options=cmd,values=None,exit_code=completed.returncode)
    if (out/'solution.raw').exists():
        lines=[line.split() for line in (out/'solution.raw').read_text().splitlines()]
        header=next(t for t in lines if t[0]=='s')
        if header[1]=='mip':
            status=header[4]
            result['status']={'o':'optimal','f':'feasible_limit','n':'infeasible','u':'undefined'}.get(status,status)
            if status in ('o','f'):
                result['objective']=float(header[5]);result['values']={names[int(t[1])]:float(t[2]) for t in lines if t[0]=='j'}
            if status=='o':result['dual_bound']=result['objective'];result['gap']=0.
            else:
                matches=re.findall(r'mip\s*=\s*([-+\deE.]+)\s*(?:>=|<=)\s*([-+\deE.]+)',completed.stdout)
                if matches:
                    result['dual_bound']=finite(float(matches[-1][1]))
                    if result['objective'] is not None:result['gap']=abs(result['objective']-result['dual_bound'])/max(1.,abs(result['objective']))
        else:result['status']='unsupported_raw_lp_status'
else:raise ValueError(solver)
(out/'result.json').write_text(json.dumps(result,allow_nan=False)+'\n')
