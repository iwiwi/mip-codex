from pathlib import Path
from fractions import Fraction as F
import json,sys
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import sha256
model=Path('artifacts/lp-primal-edge-pricing-oracle/units/root-units-failure-114.mps');root=Path('artifacts/scaled-bound-point-repair-primary-units114-paired/after');d=json.loads((root/'root-lp.json').read_text());names,variables,rows,cost,offset=exact_model(model,True);weights=[F(0)]*len(rows);assert len(d['multipliers'])==len(d['row_sources'])
for value,(i,sign)in zip(d['multipliers'],d['row_sources']):weights[i]+=F.from_float(value)*F.from_float(sign)
reduced={name:cost.get(name,F(0))for name in names};bound=offset
for y,row in zip(weights,rows):
 if y==0:continue
 side=row['lower']if y>0 else row['upper'];assert side is not None;bound+=y*side
 for name,a in row['coefficients'].items():reduced[name]-=y*a
for name,r in reduced.items():
 side=variables[name]['lower']if r>=0 else variables[name]['upper'];assert side is not None;bound+=r*side
point={k:F(str(v))for k,v in zip(d['names'],d['point'])};upper=offset+sum(a*point[k]for k,a in cost.items());result=json.loads((root/'result.json').read_text());report=F(str(result['dual_bound']));assert report<=bound;record=dict(input_sha256=sha256(model),dump_sha256=sha256(root/'root-lp.json'),result_sha256=sha256(root/'result.json'),exact_lagrangian_bound=str(bound),exact_point_objective=str(upper),reported_bound=str(report),reported_minus_exact_witness=str(report-bound),gap=str(upper-bound),reported_bound_supported=True,arithmetic='Fraction from original MPS and exact IEEE binary row multipliers; no external optimization');Path('artifacts/scaled-bound-point-repair-primary-units114-dual-audit.json').write_text(json.dumps(record,indent=2)+'\n');print(dict(reported_bound_supported=True,gap=float(upper-bound)))
