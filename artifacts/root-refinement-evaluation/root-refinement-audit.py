import json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
os.sched_setaffinity(0,set(range(33,42)))
runs=[Path(Path(f'artifacts/root-refinement-{label}.log').read_text().strip().splitlines()[-1])for label in ['smoke8','smoke10','native60']]
subprocess.run(['.venv/bin/python','scripts/audit_smoke_solutions.py','--output','artifacts/root-refinement-smoke-audit',*map(str,runs)],check=True)
records=[];out=Path('artifacts/root-refinement-main-audit');out.mkdir(exist_ok=True)
for side in ['before','after']:
 for name in ['germanrr','glass4','app1-1']:
  p=Path('artifacts/root-refinement-main60')/side/name/'result.json';meta=json.loads((p.parent/'experiment.json').read_text());model=Path(next(s for s in meta['command']if str(s).endswith('.mps.gz')));assert sha256(model)==meta['input_sha256'];r=json.loads(p.read_text());proof=validate_cut_proofs(model,r);assert proof['valid'];record=dict(source=str(p),input_sha256=sha256(model),result_sha256=sha256(p),proof=proof)
  if r.get('values')is not None:
   d=out/side/name;d.mkdir(parents=True,exist_ok=True)
   with(d/'validation.log').open('w')as stream:q=subprocess.run(['.venv/bin/python','scripts/benchmark_validate.py',str(model),str(p),'third_party/miplib-checker/bin/solchecker',str(d)],stdout=stream,stderr=subprocess.STDOUT)
   checks={k:json.loads((d/f'{k}-validation.json').read_text())for k in ['strict','official']};assert q.returncode==0 and all(v['valid']for v in checks.values());record['checks']=checks
  records.append(record)
atomic_json(out/'records.json',records);summary=dict(records=len(records),cuts=sum(r['proof']['checked']for r in records),points=sum('checks'in r for r in records));atomic_json(out/'summary.json',summary);print(json.dumps(summary))
