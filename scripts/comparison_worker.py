#!/usr/bin/env python3
"""比較専用プロセス。Rustソルバーはこのファイルを呼ばない。"""
import json
import math
import os
from pathlib import Path
import subprocess
import sys


def finite(value):
    return float(value) if math.isfinite(value) else None


def read_glpk_values(directory):
    names = {}
    with (directory/'model.glp').open() as stream:
        for line in stream:
            fields = line.split()
            if fields[:2] == ['n', 'j']:
                names[int(fields[2])] = fields[3]
    values = {}
    with (directory/'solution.raw').open() as stream:
        header = None
        for line in stream:
            fields = line.split()
            if not fields:
                continue
            if fields[0] == 's':
                header = fields
            elif fields[0] == 'j':
                if header is None:
                    raise ValueError('GLPK raw header missing')
                values[names[int(fields[1])]] = float(fields[2] if header[1] == 'mip' else fields[3])
    return values


def run(solver, filename, seconds, out):
    memory = int(os.environ.get('MIP_MEMORY_MIB', '16384'))
    if solver == 'highs':
        import highspy
        h = highspy.Highs()
        options = dict(time_limit=seconds, threads=1, random_seed=0,
                       mip_rel_gap=0., mip_abs_gap=0., small_matrix_value=1e-12)
        for key, value in options.items():
            if h.setOptionValue(key, value) != highspy.HighsStatus.kOk:
                raise ValueError('invalid HiGHS option '+key)
        h.writeOptions(str(out/'options.txt'))
        if h.readModel(filename) == highspy.HighsStatus.kError:
            raise ValueError('HiGHS reader failed')
        h.run()
        info, solution, lp = h.getInfo(), h.getSolution(), h.getLp()
        valid = solution.value_valid and info.primal_solution_status == highspy.SolutionStatus.kSolutionStatusFeasible
        objective = finite(info.objective_function_value) if valid else None
        has_integer = any(v != highspy.HighsVarType.kContinuous for v in lp.integrality_)
        dual = finite(info.mip_dual_bound) if has_integer else objective if h.getModelStatus() == highspy.HighsModelStatus.kOptimal else None
        result = dict(status=h.modelStatusToString(h.getModelStatus()),
                      objective=objective, dual_bound=dual, gap=finite(info.mip_gap) if has_integer else None,
                      nodes=info.mip_node_count if has_integer else None, seconds=h.getRunTime(),
                      version=h.version(), options=options,
                      maximize=lp.sense_ == highspy.ObjSense.kMaximize,
                      values=dict(zip(lp.col_names_, solution.col_value)) if valid else None)
    elif solver == 'scip':
        import pyscipopt
        h = pyscipopt.Model()
        h.printVersion()
        h.printExternalCodeVersions()
        options = {'limits/time': seconds, 'limits/memory': float(memory),
                   'parallel/maxnthreads': 1, 'lp/threads': 1, 'randomization/randomseedshift': 0,
                   'limits/gap': 0., 'limits/absgap': 0.}
        for key, value in options.items():
            h.setParam(key, value)
        h.writeParams(str(out/'options.txt'), comments=True, onlychanged=False)
        h.readProblem(filename)
        maximize = h.getObjectiveSense() == 'maximize'
        h.optimize()
        solution = h.getBestSol()
        def scip_number(value):
            return None if h.isInfinity(abs(value)) else finite(value)
        result = dict(status=str(h.getStatus()), objective=scip_number(h.getObjVal()) if solution is not None else None,
                      dual_bound=scip_number(h.getDualbound()), gap=scip_number(h.getGap()), nodes=h.getNNodes(),
                      seconds=h.getSolvingTime(), version=f'{h.getMajorVersion()}.{h.getMinorVersion()}.{h.getTechVersion()}',
                      options=options, maximize=maximize,
                      values={v.name: h.getSolVal(solution, v) for v in h.getVars(transformed=False)} if solution is not None else None)
    elif solver == 'glpk':
        driver = os.environ.get('MIP_GLPK_DRIVER', str(Path(__file__).resolve().parents[1]/'target/glpk-benchmark'))
        command = [driver, filename, str(seconds), str(out), str(memory)]
        completed = subprocess.run(command)
        if completed.returncode != 0:
            raise RuntimeError(f'GLPK driver exited {completed.returncode}')
        result = json.load((out/'native.json').open())
        if result['status'] == 'unknown' and result['return_code'] == 9:
            result['status'] = 'time_limit'
        result.update(nodes=None, options=dict(command=command, presolve=True, fpump=True, cuts=True, mip_gap=0.),
                      dual_bound_source='last_callback_or_optimal', gap=None,
                      values=read_glpk_values(out) if result['objective'] is not None else None)
        if result['objective'] is not None and result['dual_bound'] is not None:
            result['gap'] = abs(result['objective']-result['dual_bound'])/max(1., abs(result['objective']))
        (out/'options.txt').write_text(json.dumps(result['options'], indent=2)+'\n')
    else:
        raise ValueError('unknown comparison solver '+solver)
    with (out/'result.json').open('w') as stream:
        json.dump(result, stream, allow_nan=False)
        stream.write('\n')


if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2], float(sys.argv[3]), Path(sys.argv[4]))
