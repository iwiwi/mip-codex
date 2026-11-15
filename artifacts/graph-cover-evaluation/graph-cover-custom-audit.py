import concurrent.futures,json,os,subprocess,time,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import sha256
os.sched_setaffinity(0,set(range(33,42)));root=Path.cwd();out=root/'artifacts/graph-cover-custom-audit';out.mkdir(exist_ok=True)
for seconds,count in [(15,3),(60,2)]:
 for rep in range(count):
  flag=Path(f'artifacts/graph-cover-final-repeat{seconds}-{rep}.exit')
  while not flag.exists():time.sleep(1)
  assert flag.read_text().strip()=='0'
roots=['graph-cover-pairs','graph-cover-long-pairs','graph-cover-repeat','graph-cover-adaptive-heuristic','graph-cover-success-heuristic','graph-cover-effort-heuristic','graph-cover-final15-pairs','graph-cover-final60-pairs','graph-cover-proof-repeat','graph-cover-final-repeat15','graph-cover-final-repeat60']
paths=[p for name in roots for p in Path('artifacts',name).rglob('result.json')if(p.parent/'experiment.json').is_file()]
def check(p):
 meta=json.loads((p.parent/'experiment.json').read_text());assert meta['split']in ['development','exploration_outside_benchmark_v2'];model=Path(next(a for a in meta['command']if str(a).endswith(('.mps','.mps.gz'))));assert sha256(model)==meta['input_sha256'];dest=out/p.parent.relative_to('artifacts');dest.mkdir(parents=True,exist_ok=True)
 with(dest/'validation.log').open('w')as stream:a=subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(p),'third_party/miplib-checker/bin/solchecker',str(dest)],stdout=stream,stderr=subprocess.STDOUT)
 with(dest/'proof.log').open('w')as stream:b=subprocess.run([str(root/'.venv/bin/python'),'scripts/verify_cover_cuts.py',str(model),str(p),str(dest/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
 checks={label:json.loads((dest/f'{label}-validation.json').read_text())for label in ['strict','official']};proof=json.loads((dest/'proof.json').read_text());assert a.returncode==b.returncode==0 and all(v['valid']is True for v in checks.values())and proof['valid']is True
 return dict(source=str(p),input_sha256=sha256(model),solution_sha256=sha256(p),split=meta['split'],strict=checks['strict'],official=checks['official'],proof=proof)
with concurrent.futures.ThreadPoolExecutor(max_workers=4)as pool:records=list(pool.map(check,paths))
summary=dict(points=len(records),strict_valid=sum(r['strict']['valid']for r in records),official_valid=sum(r['official']['valid']for r in records),graph_proofs=sum(r['proof'].get('binary_graph',{}).get('valid',False)for r in records),cuts=sum(r['proof']['checked']for r in records));(out/'records.json').write_text(json.dumps(records,indent=2)+'\n');(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary),flush=True)
