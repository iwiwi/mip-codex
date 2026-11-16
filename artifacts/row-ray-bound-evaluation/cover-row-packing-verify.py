from pathlib import Path
from fractions import Fraction as F
import json,sys
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
p=Path('artifacts/cover-row-packing-cygnet');a=json.load(open(p/'stdout.json'));names,variables,rows,c,offset=exact_model('data/raw/benchmark/neos-5052403-cygnet.mps.gz',True);assert all(v['lower']==0 and c.get(n,F(0))==1 for n,v in variables.items());seen=set()
for i in a['selected_rows']:
 r=rows[i];assert r['lower']==1 and all(v==1 for v in r['coefficients'].values());assert seen.isdisjoint(r['coefficients']);seen.update(r['coefficients'])
exact=offset+len(a['selected_rows']);assert F(str(a['bound']))<=exact;out=dict(valid=True,exact_bound=str(exact),selected_rows=len(a['selected_rows']));(p/'validation.json').write_text(json.dumps(out,indent=2)+'\n');print(out)
