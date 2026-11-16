from pathlib import Path
from fractions import Fraction as F
import json,sys,struct
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
p=Path('artifacts/cover-dual-packing-cygnet');a=json.load(open(p/'stdout.json'));best=max(a['records'],key=lambda r:r['bound']);names,variables,rows,c,offset=exact_model('data/raw/benchmark/neos-5052403-cygnet.mps.gz',True);assert all(v['lower']==0 and v['upper']==1 for v in variables.values());d={n:F(0) for n in names};b=F(0);seen=set()
for i,bits in best['multipliers']:
 assert i not in seen;seen.add(i);y=F.from_float(struct.unpack('>d',bytes.fromhex(bits))[0]);assert y>=0;r=rows[i];assert r['lower'] is not None;b+=y*r['lower']
 for n,x in r['coefficients'].items():d[n]+=y*x
exact=offset+b+sum(min(F(0),c.get(n,F(0))-d[n]) for n in names);assert F(str(best['bound']))<=exact
v=dict(valid=True,trial=best['trial'],exact_bound=str(exact),claimed=best['bound'],nonzero_multipliers=len(seen));(p/'validation.json').write_text(json.dumps(v,indent=2)+'\n');print(v)
