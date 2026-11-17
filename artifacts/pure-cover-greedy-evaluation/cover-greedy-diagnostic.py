from pathlib import Path
import sys,json,os,subprocess,shutil
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,atomic_json,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-objective-repair');binary=repo/'target/release/examples/cover_greedy';info=require_solver_origin(repo,binary);out=root/'artifacts/cover-greedy-diagnostic';out.mkdir();(out/'bin').mkdir();shutil.copy2(binary,out/'bin/helper');shutil.copy2(repo/'examples/cover_greedy.rs',out/'helper.rs');atomic_json(out/'experiment.json',dict(cpu=39,build_info=info,binary_sha256=sha256(binary),source_reference='artifacts/component-work-share-default-on-diagnostic',purpose='standalone development heuristic diagnostic; extra budget, not whole-solver performance'));os.sched_setaffinity(0,{39});manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for name in ['fast0507.mps.gz','glass-sc.mps.gz']:
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256']
 for seconds in [.1,.5,2.]:
  d=out/name/str(seconds);d.mkdir(parents=True);cmd=[str(out/'bin/helper'),str(model),str(seconds),str(d/'result.json')];atomic_json(d/'job.json',dict(command=cmd,input_sha256=r['sha256']))
  with(d/'stdout.json').open('w')as stream,(d/'stderr.log').open('w')as err:proc=subprocess.run(cmd,stdout=stream,stderr=err,timeout=60)
  assert proc.returncode==0;v=json.load(open(d/'stdout.json'));records.append(dict(name=name,budget=seconds,**v));print(name,seconds,v,flush=True)
atomic_json(out/'records.json',records);atomic_json(out/'exit.json',dict(status='complete'))
