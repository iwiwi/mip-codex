import json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json
from verify_cover_cuts import validate_cut_proofs
os.sched_setaffinity(0,set(range(33,42)))
runs=[Path(Path(f'artifacts/aggregate-row512-smoke{n}.log').read_text().strip().splitlines()[-1])for n in [8,16]]
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/aggregate-row512-smoke-audit',*map(str,runs)],check=True)
records=[]
for seconds in [10,60]:
 for side in ['before','after']:
  for name in ['timtab1','fhnw-binpack4-4']:
   p=Path(f'artifacts/aggregate-row512-pairs{seconds}')/side/name/'result.json';meta=json.loads((p.parent/'experiment.json').read_text());assert meta['split']=='development';model=Path(next(a for a in meta['command']if str(a).endswith('.mps.gz')));assert sha256(model)==meta['input_sha256']
   r=json.loads(p.read_text());proof=validate_cut_proofs(model,r);assert proof['valid'];records.append(dict(source=str(p),solution_sha256=sha256(p),input_sha256=sha256(model),proof=proof))
atomic_json(Path('artifacts/aggregate-row512-pair-audit.json'),records);print(json.dumps(dict(records=len(records),valid=len(records),cuts=sum(r['proof']['checked']for r in records))))
