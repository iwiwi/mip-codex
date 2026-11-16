from pathlib import Path
import json,sys,struct
from fractions import Fraction as F
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import sha256
r=Path.cwd();source=r/'artifacts/priced-dual-subgradient-short-diagnostic';records=[]
for folder in sorted(p for p in source.iterdir()if p.is_dir()and p.name.endswith('.mps.gz')):
 model=r/'data/raw/benchmark'/folder.name;names,variables,rows,c,offset=exact_model(str(model),True);assert all(v['lower']==0 and v['upper']==1 for v in variables.values())
 for p in sorted(folder.glob('*/stdout.json')):
  a=json.load(open(p));d={n:F(0)for n in names};b=offset;seen=set()
  for i,bits in a['multipliers']:
   assert i not in seen;seen.add(i);y=F.from_float(struct.unpack('>d',bytes.fromhex(bits))[0]);assert y>=0;row=rows[i];assert row['lower']is not None;b+=y*row['lower']
   for n,x in row['coefficients'].items():d[n]+=y*x
  exact=b+sum(min(F(0),c.get(n,F(0))-d[n])for n in names);assert a['certified_bound']is not None and F(str(a['certified_bound']))<=exact;record=dict(source=str(p),source_sha256=sha256(p),model_sha256=sha256(model),valid=True,exact_bound=str(exact),reported=a['certified_bound']);(p.parent/'validation.json').write_text(json.dumps(record,indent=2)+'\n');records.append(record)
(source/'validation-summary.json').write_text(json.dumps(dict(valid=len(records),records=records),indent=2)+'\n');print(len(records))
