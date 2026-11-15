#!/usr/bin/env python3
"""Independent original-MPS solution check using HiGHS only as an MPS reader.
No optimization is run. Requires the comparison environment's highspy/numpy.
"""
import argparse, json, math
import highspy
p=argparse.ArgumentParser();p.add_argument('mps');p.add_argument('solution');p.add_argument('--tolerance',type=float,default=1e-7);args=p.parse_args()
h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('log_to_console',False)
assert h.readModel(args.mps)==highspy.HighsStatus.kOk
lp=h.getLp();sol=json.load(open(args.solution));x=sol.get('values')
if x is None: raise SystemExit('no incumbent to verify')
assert len(x)==lp.num_col_ and all(math.isfinite(v) for v in x)
# CLI emits original MPS column order; also verify names when available.
if sol.get('variable_names') is not None:
 names=sol['variable_names'];assert len(names)==len(set(names))==len(x)
 byname=dict(zip(names,x));assert set(byname)==set(lp.col_names_)
 x=[byname[name] for name in lp.col_names_]
tol=args.tolerance
bound=max([0.]+[max(l-v,v-u) for l,v,u in zip(lp.col_lower_,x,lp.col_upper_)])
integer=max([0.]+[abs(v-round(v)) for v,t in zip(x,lp.integrality_) if t==highspy.HighsVarType.kInteger])
assert all(t in (highspy.HighsVarType.kContinuous,highspy.HighsVarType.kInteger) for t in lp.integrality_)
a=lp.a_matrix_;terms=[[] for _ in range(lp.num_row_)]
if a.format_==highspy.MatrixFormat.kColwise:
 for j in range(lp.num_col_):
  for k in range(a.start_[j],a.start_[j+1]):terms[a.index_[k]].append(a.value_[k]*x[j])
else:
 for i in range(lp.num_row_):
  terms[i]=[a.value_[k]*x[a.index_[k]] for k in range(a.start_[i],a.start_[i+1])]
activity=[math.fsum(row) for row in terms]
row=max([0.]+[max(l-v,v-u) for l,v,u in zip(lp.row_lower_,activity,lp.row_upper_)])
objective=lp.offset_+math.fsum(c*v for c,v in zip(lp.col_cost_,x));error=abs(objective-sol['objective'])
valid=bool(bound<=tol and integer<=tol and row<=tol and error<=tol*max(1,abs(objective)))
print(json.dumps(dict(valid=valid,objective=objective,max_bound_violation=bound,max_row_violation=row,max_integrality_violation=integer,objective_error=error)))
raise SystemExit(0 if valid else 1)
