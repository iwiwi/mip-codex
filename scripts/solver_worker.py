#!/usr/bin/env python3
"""Comparison solvers only. Never called from Rust optimization code."""
import json,math,pathlib,subprocess,sys,tempfile,time
solver,path,limit,out=sys.argv[1:];limit=float(limit);start=time.monotonic()
def finite(x):return float(x) if math.isfinite(x) and abs(x)<1e30 else None
if solver=='highs':
 import highspy
 h=highspy.Highs();h.setOptionValue('output_flag',False)
 for k,v in [('threads',1),('parallel','off'),('random_seed',0),('mip_rel_gap',1e-6),('time_limit',limit)]:h.setOptionValue(k,v)
 assert h.readModel(path)==highspy.HighsStatus.kOk
 h.run();info=h.getInfo();s=h.getSolution();lp=h.getLp();has_incumbent=info.primal_solution_status==highspy.SolutionStatus.kSolutionStatusFeasible
 result=dict(status=h.modelStatusToString(h.getModelStatus()),objective=finite(info.objective_function_value) if has_incumbent else None,dual_bound=finite(info.mip_dual_bound),nodes=info.mip_node_count,values=list(s.col_value) if has_incumbent else None,variable_names=lp.col_names_,solver_seconds=h.getRunTime())
elif solver=='scip':
 import pyscipopt
 h=pyscipopt.Model();h.hideOutput()
 for k,v in [('parallel/maxnthreads',1),('lp/threads',1),('randomization/randomseedshift',0),('limits/gap',1e-6),('limits/time',limit)]:h.setParam(k,v)
 h.readProblem(path);variables=h.getVars(transformed=False);h.optimize();s=h.getBestSol()
 result=dict(status=str(h.getStatus()),objective=finite(h.getObjVal()) if s else None,dual_bound=finite(h.getDualbound()),nodes=h.getNNodes(),values=[h.getSolVal(s,v) for v in variables] if s else None,variable_names=[v.name for v in variables],solver_seconds=h.getSolvingTime())
elif solver=='glpk':
 driver=pathlib.Path(__file__).resolve().parents[1]/'tools/glpk_driver'
 subprocess.run([str(driver),path,str(limit),out],check=True)
 result=json.loads(pathlib.Path(out).read_text())
else:raise SystemExit('unknown solver')
result['worker_seconds']=time.monotonic()-start
pathlib.Path(out).write_text(json.dumps(result,allow_nan=False)+'\n')
