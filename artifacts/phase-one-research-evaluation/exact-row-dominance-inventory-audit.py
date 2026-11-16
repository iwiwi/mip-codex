from pathlib import Path
import json,sys,os
sys.path.insert(0,'/home/ubuntu/mip-dual-restoration/scripts');from verify_row_dominance import validate_row_dominance
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{37});root=Path.cwd();source=root/'artifacts/exact-row-dominance-inventory-v1';records=[];manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']}
for p in sorted(source.glob('*.mps.gz/result.json')):
 r=manifest[p.parent.name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];v=json.load(open(p));check=validate_row_dominance(model,dict(row_dominance_proofs=v['proofs']));assert check['valid'];record=dict(name=r['name'],input_sha256=r['sha256'],proof_sha256=sha256(p),validation=check);records.append(record);print(record['name'],check,flush=True);atomic_json(source/'proof-audit-progress.json',dict(completed=len(records),proofs=sum(r['validation']['checked']for r in records)))
atomic_json(source/'proof-audit.json',dict(records=records,valid=True,proofs=sum(r['validation']['checked']for r in records)))
