from pathlib import Path
import json,subprocess,sys,shutil
sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-priced-repair');out=root/'artifacts/priced-primal-repair-v1-diagnostic';out.mkdir();(out/'bin').mkdir();binary=repo/'target/release/examples/priced_repair';info=require_solver_origin(repo,binary);shutil.copy2(binary,out/'bin/priced_repair')
for n in subprocess.check_output(['git','ls-files','src','scripts','tests','examples','Cargo.toml','Cargo.lock','build.rs'],cwd=repo,text=True).splitlines()+['examples/priced_repair.rs','tests/priced_repair.rs']:
 q=out/'source'/n;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(repo/n,q)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),seconds=.5,cpu=42),indent=2));records=[]
for name in ['sp97ar.mps.gz','sp98ar.mps.gz','tbfp-network.mps.gz','neos-5052403-cygnet.mps.gz']:
 seed=root/'artifacts/root-seed-repair-final-paired-41'/name/'after/result.json'
 if not seed.exists():
  seed=root/'artifacts/root-objective-repair-v1-paired-41'/name/'before/result.json'
 assert seed.exists(),seed
 for weight in ['0.1','1','10']:
  d=out/name/weight;d.mkdir(parents=True);model=root/'data/raw/benchmark'/name;(d/'input.json').write_text(json.dumps(dict(model=str(model),model_sha256=sha256(model),seed=str(seed),seed_sha256=sha256(seed)),indent=2));cmd=['taskset','-c','42','timeout','--signal=KILL','65',str(out/'bin/priced_repair'),str(model),str(seed),'0.5',weight,str(d/'solution.json')]
  with(d/'stdout.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:r=subprocess.run(cmd,stdout=stdout,stderr=stderr)
  assert r.returncode==0;a=json.load(open(d/'stdout.json'));assert a['source_fingerprint']==info['source_fingerprint'];records.append(dict(name=name,**a));(out/'summary.json').write_text(json.dumps(records,indent=2))
print(records)
