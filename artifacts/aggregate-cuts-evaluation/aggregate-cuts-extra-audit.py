import concurrent.futures,json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json
os.sched_setaffinity(0,set(range(33,42)));root=Path.cwd();out=root/'artifacts/aggregate-cuts-extra-audit2';out.mkdir(exist_ok=True);jobs=[]
for p in Path('artifacts/aggregate-cut-roots').glob('*/result.json'):
 if json.loads(p.read_text()).get('values')is not None:
  meta=json.loads((p.parent/'experiment.json').read_text());model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];jobs.append((p,model))
for i in [0,1]:jobs.append((Path(f'artifacts/aggregate-cuts-toy/{i}.json'),Path('artifacts/aggregate-cuts-toy/model.mps')))
def check(pair):
 p,model=pair;dest=out/p.relative_to('artifacts');dest.mkdir(parents=True,exist_ok=True)
 with(dest/'validation.log').open('w')as stream:a=subprocess.run(['.venv/bin/python','scripts/benchmark_validate.py',str(model),str(p),'third_party/miplib-checker/bin/solchecker',str(dest)],stdout=stream,stderr=subprocess.STDOUT)
 with(dest/'proof.log').open('w')as stream:b=subprocess.run(['.venv/bin/python','scripts/verify_cover_cuts.py',str(model),str(p),str(dest/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
 strict=json.loads((dest/'strict-validation.json').read_text());official=json.loads((dest/'official-validation.json').read_text());proof=json.loads((dest/'proof.json').read_text());assert a.returncode==b.returncode==0 and strict['valid']and official['valid']and proof['valid']
 return dict(source=str(p),input_sha256=sha256(model),result_sha256=sha256(p),strict=strict,official=official,proof=proof)
with concurrent.futures.ThreadPoolExecutor(max_workers=4)as pool:records=list(pool.map(check,jobs))
atomic_json(out/'records.json',records);summary=dict(points=len(records),strict_valid=sum(r['strict']['valid']for r in records),official_valid=sum(r['official']['valid']for r in records),proofs=len(records),cuts=sum(r['proof']['checked']for r in records));atomic_json(out/'summary.json',summary);print(json.dumps(summary))
