import json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model,verify_mir_exact
from benchmark_common import atomic_json,sha256
records=[]
for p in sorted(Path('artifacts/mir-diagnostic').glob('*.json')):
 data=json.loads(p.read_text())
 if 'cuts'not in data or 'model'not in data:continue
 names,variables,rows=exact_model(data['model']);count=0
 for c in data['cuts']:
  old=c['proof'];proof={k:old[k]for k in ['source_row','source_name','upper_side','divisor']};proof.update(cover=[],mir={'upper_bounds':[b['variable']for b in old['bounds']if b['upper']]})
  cut={'proof':proof,'row':{'coefficients':c['coefficients'],'lower':None,'upper':c['upper'],'exact_integral':True}}
  verify_mir_exact(names,variables,rows[proof['source_row']],cut);count+=1
 records.append(dict(name=data['name'],checked=count,input_sha256=sha256(data['model'])))
atomic_json(Path('artifacts/mir-diagnostic-fraction-audit.json'),dict(valid=True,checked=sum(r['checked']for r in records),models=len(records),records=records))
print(json.dumps(dict(valid=True,checked=sum(r['checked']for r in records),models=len(records))))
