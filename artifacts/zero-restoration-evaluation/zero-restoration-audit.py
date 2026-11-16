import json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
os.sched_setaffinity(0,set(range(33,42)))
labels=['smoke8','smoke10','specialized-smoke8','specialized-smoke10','native60']
runs=[Path(Path(f'artifacts/zero-restoration-{label}.log').read_text().strip().splitlines()[-1])for label in labels]
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/zero-restoration-smoke-audit',*map(str,runs)],check=True)
roots=['zero-restoration-main60','zero-restoration-mode-main60','zero-restoration-mode-neos60','zero-restoration-specialized-neos60','zero-restoration-repeat10','zero-restoration-specialized-repeat10']
paths=[p for name in roots for p in Path('artifacts',name).rglob('result.json')if(p.parent/'experiment.json').is_file()]
out=Path('artifacts/zero-restoration-extra-audit');out.mkdir(exist_ok=True);records=[]
for p in paths:
 meta=json.loads((p.parent/'experiment.json').read_text());model=Path(next(s for s in meta['command']if str(s).endswith('.mps.gz')));assert meta['split']=='development'and sha256(model)==meta['input_sha256'];r=json.loads(p.read_text());proof=validate_cut_proofs(model,r);assert proof['valid'];record=dict(source=str(p),input_sha256=sha256(model),result_sha256=sha256(p),proof=proof)
 if r.get('values')is not None:
  d=out/p.parent.relative_to('artifacts');d.mkdir(parents=True,exist_ok=True)
  with(d/'validation.log').open('w')as stream:q=subprocess.run(['.venv/bin/python','scripts/benchmark_validate.py',str(model),str(p),'third_party/miplib-checker/bin/solchecker',str(d)],stdout=stream,stderr=subprocess.STDOUT)
  checks={k:json.loads((d/f'{k}-validation.json').read_text())for k in ['strict','official']};assert q.returncode==0 and all(v['valid']for v in checks.values());record['checks']=checks
 records.append(record)
atomic_json(out/'records.json',records);summary=dict(records=len(records),cuts=sum(r['proof']['checked']for r in records),points=sum('checks'in r for r in records));atomic_json(out/'summary.json',summary);print(json.dumps(summary))
