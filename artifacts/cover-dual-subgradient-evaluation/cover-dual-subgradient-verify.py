from pathlib import Path
import json,sys,struct
from fractions import Fraction as F
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
names,variables,rows,c,offset=exact_model('data/raw/benchmark/neos-5052403-cygnet.mps.gz',True);assert all(v['lower']==0 and v['upper']==1 for v in variables.values());files=[Path('artifacts/cover-dual-subgradient-cygnet/stdout.json'),*sorted(Path('artifacts/cover-dual-subgradient-v2').glob('*/stdout.json'))];records=[]
for p in files:
 a=json.load(open(p));d={n:F(0)for n in names};b=offset;seen=set()
 for i,bits in a['multipliers']:
  assert i not in seen;seen.add(i);y=F.from_float(struct.unpack('>d',bytes.fromhex(bits))[0]);assert y>=0;r=rows[i];assert r['lower']is not None;b+=y*r['lower']
  for n,x in r['coefficients'].items():d[n]+=y*x
 exact=b+sum(min(F(0),c.get(n,F(0))-d[n])for n in names);assert F(str(a['certified_bound']))<=exact;record=dict(source=str(p),valid=True,exact_bound=str(exact),reported=a['certified_bound']);(p.parent/'validation.json').write_text(json.dumps(record,indent=2));records.append(record)
Path('artifacts/cover-dual-subgradient-validation.json').write_text(json.dumps(dict(records=records,valid=len(records)),indent=2));print(len(records))
