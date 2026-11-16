import json,sys,os
from pathlib import Path
from decimal import Decimal
from fractions import Fraction as F
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{38});directory=Path('artifacts/mir-matrix-neos-root');meta=json.loads((directory/'experiment.json').read_text());model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];data=json.loads((directory/'root-lp.json').read_text(),parse_float=Decimal);names,variables,rows,cost,offset=exact_model(model,True);assert data['names']==names;point={n:F(x)for n,x in zip(names,data['point'])};bv=F(0);rv=F(0)
for n,v in variables.items():
 if v['lower']is not None:bv=max(bv,v['lower']-point[n])
 if v['upper']is not None:bv=max(bv,point[n]-v['upper'])
for r in rows:
 a=sum((c*point[n]for n,c in r['coefficients'].items()),F(0))
 if r['lower']is not None:rv=max(rv,r['lower']-a)
 if r['upper']is not None:rv=max(rv,a-r['upper'])
obj=offset+sum((c*point[n]for n,c in cost.items()),F(0));error=abs(obj-F(data['objective']));result=dict(valid=bv<=F(1,10**6)and rv<=F(1,10**6)and error<=F(1,10**8)*(1+abs(obj)),purpose='LP point only, no integer feasibility or dual/optimality certification',bound_violation=str(bv),row_violation=str(rv),objective_error=str(error),exact_objective=str(obj),input_sha256=sha256(model),point_sha256=sha256(directory/'root-lp.json'));atomic_json(directory/'lp-audit.json',result);print(json.dumps(result));assert result['valid']
