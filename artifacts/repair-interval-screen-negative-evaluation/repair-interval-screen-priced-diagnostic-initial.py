from pathlib import Path
import json,subprocess,shutil,sys,os
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
root=Path.cwd();repo=Path('/home/ubuntu/mip-priced-repair');binary=repo/'target/release/examples/priced_repair';info=require_solver_origin(repo,binary);out=root/'artifacts/repair-interval-screen-priced-diagnostic';out.mkdir();(out/'bin').mkdir();shutil.copy2(binary,out/'bin/helper');shutil.copy2(repo/'examples/priced_repair.rs',out/'helper.rs');atomic_json(out/'experiment.json',dict(cpu=37,seconds=10,work_budget=100_000_000,build_info=info,binary_sha256=sha256(binary),source_reference='artifacts/repair-interval-screen-v1-diagnostic'));os.sched_setaffinity(0,{37});manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for i,name in enumerate(['n3div36.mps.gz','sp97ar.mps.gz','sp98ar.mps.gz']):
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];seed=root/'artifacts/repair-check-screen-v1-paired-41'/name/'before/result.json';assert seed.exists()
 for weight in [1.,.3]:
  pair=[]
  for label in (['before','after']if i%2==0 else['after','before']):
   d=out/name/str(weight)/label;d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_REPAIR_INTERVAL_SCREEN']='0'if label=='before'else'1';cmd=[str(out/'bin/helper'),str(model),str(seed),'10',str(weight),str(d/'result.json')];atomic_json(d/'job.json',dict(command=cmd,input_sha256=r['sha256'],seed_sha256=sha256(seed),flag=env['MIP_REPAIR_INTERVAL_SCREEN']))
   with(d/'stdout.json').open('w')as stream,(d/'stderr.log').open('w')as err:proc=subprocess.run(cmd,stdout=stream,stderr=err,env=env,timeout=30)
   assert proc.returncode==0;v=json.load(open(d/'stdout.json'));record=dict(name=name,label=label,**v);records.append(record);pair.append(record);print(json.dumps(record),flush=True)
  atomic_json(out/name/str(weight)/'comparison.json',dict(same_work=pair[0]['work']==pair[1]['work'],same_moves=pair[0]['moves']==pair[1]['moves'],same_objective=pair[0]['objective']==pair[1]['objective']))
atomic_json(out/'records.json',records);atomic_json(out/'exit.json',dict(status='complete'))
