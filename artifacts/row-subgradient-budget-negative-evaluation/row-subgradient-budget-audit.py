import json,sys,os
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
root=Path.cwd();o=root/'artifacts/row-subgradient-budget-diagnostic';assert (o/'exit.json').exists();m={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};records=[];os.sched_setaffinity(0,{39})
for p in sorted(o.glob('*/*/result.json')):
 name=p.parent.parent.name;r=m[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];v=json.loads(p.read_text());a=validate_cut_proofs(model,v);assert a['valid']is True;record=dict(name=name,seconds=float(p.parent.name),result_sha256=sha256(p),dual_bound=v['dual_bound'],proof=a);records.append(record);print(name,p.parent.name,v['dual_bound'],flush=True)
assert len(records)==16;atomic_json(o/'proof-audit.json',dict(records=records,valid=len(records),checker_sha256=sha256(root/'scripts/verify_cover_cuts.py')))
