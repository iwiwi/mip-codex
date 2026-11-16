from pathlib import Path
import os,sys,json,subprocess
sys.path.insert(0,'scripts');from benchmark_common import atomic_json
root=Path.cwd();os.sched_setaffinity(0,{38});out=root/'artifacts/partial-phase1-generated-audit';out.mkdir(exist_ok=False);records=[]
for f in sorted((root/'artifacts/partial-phase1-thin-generated').glob('*/*/result.json')):
 model=f.parent.parent/'model.mps';d=out/f.parent.parent.name/f.parent.name;d.mkdir(parents=True)
 with (d/'audit.log').open('w')as log:p=subprocess.run([sys.executable,'scripts/benchmark_validate.py',str(model),str(f),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=log,stderr=subprocess.STDOUT,timeout=120)
 r=dict(case=f.parent.parent.name,variant=f.parent.name,exit_code=p.returncode,**{k:json.load(open(d/(k+'-validation.json')))for k in ['strict','official']});records.append(r);atomic_json(d/'record.json',r)
summary=dict(points=len(records),valid=sum(all(r[k]['valid']is True for k in ['strict','official'])for r in records));atomic_json(out/'records.json',records);atomic_json(out/'summary.json',summary);print(summary);assert summary['points']==summary['valid']==12
