import json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import validate_cut_proofs
from benchmark_common import atomic_json
records=[]
for p in sorted(Path('artifacts/mir-rust-diagnostic').glob('*.json')):
 d=json.loads(p.read_text());audit=validate_cut_proofs(d['model'],d);assert audit['valid'],(p,audit)
 records.append(dict(name=d['name'],cuts=len(d['cut_proofs']),work=d['cut_stats']['mir_work'],seconds=d['cut_stats']['mir_seconds'],validation=audit))
atomic_json(Path('artifacts/mir-rust-diagnostic-audit.json'),dict(valid=True,models=len(records),positive_models=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records),records=records))
print(json.dumps(dict(valid=True,models=len(records),positive_models=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records))))
print(json.dumps([{k:r[k]for k in ['name','cuts','work','seconds']}for r in records if r['cuts']]))
