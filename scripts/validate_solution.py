#!/usr/bin/env python3
"""元MPSを独立したHiGHS readerで読んで検査。最適化APIは呼ばない。
Usage: .venv/bin/python scripts/validate_solution.py original.mps.gz solution.json
解形式: {"objective": number, "values": {"variable_name": number, ...}}
全変数の明示が必要。検査は最適性証明ではない。
"""
import json
import math
import sys
import highspy
import numpy as np
from scipy.sparse import csc_matrix


def original_lp(path):
    h = highspy.Highs()
    h.setOptionValue('output_flag', False)
    h.setOptionValue('small_matrix_value', 1e-12)
    if h.readModel(str(path)) == highspy.HighsStatus.kError:
        raise ValueError('independent MPS reader failed')
    return h.getLp()


def matrix(lp):
    a = lp.a_matrix_
    if a.format_ != highspy.MatrixFormat.kColwise:
        raise ValueError('expected column matrix')
    return csc_matrix((a.value_, a.index_, a.start_), shape=(lp.num_row_, lp.num_col_))


def validate(path, solution, feasibility=1e-6, integrality=1e-6):
    lp = original_lp(path)
    values = solution['values']
    if len(set(lp.col_names_)) != lp.num_col_ or set(values) != set(lp.col_names_):
        raise ValueError('solution variable names do not exactly match original problem')
    x = np.array([values[n] for n in lp.col_names_], dtype=float)
    if not np.isfinite(x).all() or not math.isfinite(solution['objective']):
        raise ValueError('nonfinite solution')
    bound_violation = max(0., float(np.max(np.asarray(lp.col_lower_)-x, initial=0.)),
                          float(np.max(x-np.asarray(lp.col_upper_), initial=0.)))
    integer_violation = 0.
    if lp.integrality_:
        mask = np.array([v == highspy.HighsVarType.kInteger for v in lp.integrality_])
        if any(v not in (highspy.HighsVarType.kInteger, highspy.HighsVarType.kContinuous) for v in lp.integrality_):
            raise ValueError('unsupported variable domain in validator')
        integer_violation = float(np.max(np.abs(x[mask]-np.rint(x[mask])), initial=0.))
    a = matrix(lp).tocsr()
    row_violation = 0.
    for i in range(lp.num_row_):
        start, end = a.indptr[i:i+2]
        activity = math.fsum(float(v)*float(x[j]) for v,j in zip(a.data[start:end], a.indices[start:end]))
        if not math.isfinite(activity):
            raise ValueError('nonfinite row activity')
        row_violation = max(row_violation, lp.row_lower_[i]-activity, activity-lp.row_upper_[i])
    objective = math.fsum([lp.offset_, *(float(c)*float(v) for c,v in zip(lp.col_cost_,x))])
    objective_error = abs(objective-solution['objective'])
    valid = (bound_violation <= feasibility and row_violation <= feasibility
             and integer_violation <= integrality and objective_error <= 1e-8*(1+abs(objective)))
    return dict(valid=valid, objective=objective, bound_violation=bound_violation,
                row_violation=row_violation, integer_violation=integer_violation,
                objective_error=objective_error, feasibility_tolerance=feasibility,
                integrality_tolerance=integrality, reader='HiGHS '+'.'.join(str(getattr(highspy,'HIGHS_VERSION_'+k)) for k in ('MAJOR','MINOR','PATCH')))


if __name__ == '__main__':
    try:
        result = validate(sys.argv[1], json.load(open(sys.argv[2])))
        print(json.dumps(result, allow_nan=False))
        sys.exit(0 if result['valid'] else 1)
    except (ValueError, KeyError, TypeError, IndexError, OverflowError) as error:
        print(json.dumps(dict(valid=False, error=str(error))))
        sys.exit(1)
