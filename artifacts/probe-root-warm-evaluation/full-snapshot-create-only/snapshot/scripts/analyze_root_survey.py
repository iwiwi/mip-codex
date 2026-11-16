#!/usr/bin/env python3
"""Independent rational primal checks of diagnostic LP points, no optimization."""
import argparse,json,os
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
from benchmark_common import atomic_json,sha256
from verify_cover_cuts import exact_model
from verify_binary_graph import maximize_sense
p=argparse.ArgumentParser();p.add_argument('directory',type=Path);p.add_argument('--partition',type=int,required=True);p.add_argument('--partitions',type=int,default=4);p.add_argument('--cpu',type=int,required=True);args=p.parse_args()
assert 0<=args.partition<args.partitions;os.sched_setaffinity(0,{args.cpu})
folders=sorted(d.parent for d in args.directory.glob('*/summary.json'));assert len(folders)==188
records=[]
for k,d in enumerate(folders):
 if k%args.partitions!=args.partition:continue
 summary=json.loads((d/'summary.json').read_text());meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development'
 record=dict(name=summary['name'],solver_status=summary.get('status'),point_dump=summary['point_dump'],purpose='LP primal diagnostic only; not integer feasibility or dual certification',valid=None)
 if summary['point_dump']:
  try:
   model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];data=json.loads((d/'root-lp.json').read_text(),parse_float=Decimal)
   names,variables,rows,cost,offset=exact_model(model,True);assert names==data['names']and len(names)==len(data['point'])
   point={name:Fraction(v)for name,v in zip(names,data['point'])};bound_violation=Fraction(0);row_violation=Fraction(0);worst_row=None;fractional=0
   for name,v in variables.items():
    x=point[name]
    if v['lower']is not None:bound_violation=max(bound_violation,v['lower']-x)
    if v['upper']is not None:bound_violation=max(bound_violation,x-v['upper'])
    if v['integer']and min(x-x.numerator//x.denominator,1-(x-x.numerator//x.denominator))>Fraction(1,10**6):fractional+=1
   for row in rows:
    activity=sum((a*point[name]for name,a in row['coefficients'].items()),Fraction(0));violation=Fraction(0)
    if row['lower']is not None:violation=max(violation,row['lower']-activity)
    if row['upper']is not None:violation=max(violation,activity-row['upper'])
    if violation>row_violation:row_violation=violation;worst_row=row['name']
   objective=offset+sum((a*point[name]for name,a in cost.items()),Fraction(0));tol=Fraction(1,10**8)*(1+abs(objective))
   reported=data.get('objective');objective_error=None if reported is None else abs(Fraction(reported)-objective)
   result=json.loads((d/'result.json').read_text(),parse_float=Decimal);dual=result.get('dual_bound');maximize=maximize_sense(model)
   apparent_excess=None if dual is None else (objective-Fraction(dual)if maximize else Fraction(dual)-objective)
   valid=bound_violation<=Fraction(1,10**6)and row_violation<=Fraction(1,10**6)
   record.update(valid=valid,input_sha256=meta['input_sha256'],root_dump_sha256=sha256(d/'root-lp.json'),checker_sha256=sha256(Path(__file__)),variables=len(names),rows=len(rows),exact_objective=str(objective),bound_violation=str(bound_violation),row_violation=str(row_violation),worst_row=worst_row,fractional_integer_variables=fractional,objective_error=None if objective_error is None else str(objective_error),objective_report_within_relative_1e8=None if objective_error is None else objective_error<=tol,apparent_dual_excess=None if apparent_excess is None else str(apparent_excess),apparent_dual_excess_over_tolerance=None if apparent_excess is None else apparent_excess>tol,exactly_feasible=bound_violation==row_violation==0)
  except Exception as error:record['error']=f'{type(error).__name__}: {error}'
 else:record['reason']='no completed root LP point'
 atomic_json(d/'lp-audit.json',record);records.append(record);print(json.dumps(record),flush=True)
atomic_json(args.directory/f'analysis-partition-{args.partition}.json',dict(records=records,valid=sum(r['valid']is True for r in records),invalid=sum(r['valid']is False for r in records),unverified=sum(r['valid']is None for r in records)))
