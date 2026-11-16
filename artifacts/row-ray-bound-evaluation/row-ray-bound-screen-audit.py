from pathlib import Path
from fractions import Fraction as F
import json,struct,sys,concurrent.futures as cf
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from verify_binary_graph import maximize_sense
root=Path.cwd();out=root/'artifacts/row-ray-bound-development';records=[]
def check(p):
 a=json.load(open(p/'stdout.json'))
 if not a['supported']:return None
 path=root/'data/raw/benchmark'/p.name;names,variables,rows,c,offset=exact_model(path,True);sign=-1 if maximize_sense(path) else 1
 assert all(v['lower']==0 and v['upper']==1 for v in variables.values());y=F.from_float(struct.unpack('>d',bytes.fromhex(a['multiplier_bits']))[0]);assert y>=0
 indices=a['rows'];assert indices==sorted(set(indices));d={n:F(0) for n in names};b=F(0)
 for i in indices:
  r=rows[i];assert r['lower'] is not None and r['lower']>0 and all(x>=0 for x in r['coefficients'].values());b+=r['lower']
  for n,x in r['coefficients'].items():d[n]+=x
 exact=sign*offset+y*b+sum(min(F(0),sign*c.get(n,F(0))-y*d[n]) for n in names)
 claimed=F.from_float(struct.unpack('>d',bytes.fromhex(a['normalized_bound_bits']))[0]);assert claimed<=exact
 result=dict(name=p.name,valid=True,exact_bound=str(exact),certified_bound=float(claimed));(p/'validation.json').write_text(json.dumps(result));return result
with cf.ThreadPoolExecutor(2) as pool:
 for a in pool.map(check,sorted(p for p in out.iterdir() if p.is_dir() and (p/'execution.json').exists())):
  if a:records.append(a);(out/'audit-progress.json').write_text(json.dumps(records,indent=2))
(out/'audit-summary.json').write_text(json.dumps(dict(checked=len(records),valid=len(records),records=records),indent=2));print(len(records),'valid')
