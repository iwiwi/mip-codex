from pathlib import Path
from fractions import Fraction as F
import json,sys,struct
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
p=Path('artifacts/row-ray-bound-diagnostic');a=json.load(open(p/'cygnet.json'))
names,variables,rows,c,offset=exact_model('data/raw/benchmark/neos-5052403-cygnet.mps.gz',True)
assert all(v['lower']==0 and v['upper']==1 for v in variables.values())
y=F.from_float(struct.unpack('>d',bytes.fromhex(a['multiplier_bits']))[0]);assert y>=0
indices=a['rows'];assert indices==sorted(set(indices));d={n:F(0) for n in names};b=F(0)
for i in indices:
 r=rows[i];assert r['lower'] is not None and r['lower']>0 and all(x>=0 for x in r['coefficients'].values());b+=r['lower']
 for n,x in r['coefficients'].items():d[n]+=x
exact=offset+y*b+sum(min(F(0),c.get(n,F(0))-y*d[n]) for n in names)
claimed=F.from_float(struct.unpack('>d',bytes.fromhex(a['normalized_bound_bits']))[0]);assert claimed<=exact
out=dict(valid=True,exact_bound=str(exact),certified_bound=float(claimed),selected_rows=len(indices),multiplier=str(y));(p/'cygnet-validation.json').write_text(json.dumps(out,indent=2)+'\n');print(out)
