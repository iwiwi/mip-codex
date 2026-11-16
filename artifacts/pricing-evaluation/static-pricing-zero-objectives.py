import json,os,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{37});records=[]
for p in sorted(Path('artifacts/pricing-baseline-root-survey').glob('*/experiment.json')):
 meta=json.loads(p.read_text());model=Path(meta['command'][2]);assert meta['split']=='development'and sha256(model)==meta['input_sha256'];names,variables,rows,cost,offset=exact_model(model,True);nonzero=sum(c!=0 for c in cost.values());record=dict(name=p.parent.name,nonzero_costs=nonzero,constant_objective=nonzero==0,offset=str(offset),input_sha256=sha256(model));records.append(record)
 if nonzero==0:print(json.dumps(record),flush=True)
atomic_json(Path('artifacts/static-pricing-zero-objectives.json'),records)
