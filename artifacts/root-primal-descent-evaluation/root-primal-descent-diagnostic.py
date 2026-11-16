from pathlib import Path
import subprocess,json,sys,shutil
sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/root-primal-descent-diagnostic';out.mkdir();binary=repo/'target/release/examples/primal_descent';info=require_solver_origin(repo,binary);(out/'bin').mkdir();shutil.copy2(binary,out/'bin/primal_descent');records=[]
for n in subprocess.check_output(['git','ls-files','src','Cargo.toml','Cargo.lock','build.rs'],cwd=repo,text=True).splitlines()+['examples/primal_descent.rs','tests/primal_descent.rs']:
 q=out/'source'/n;q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(repo/n,q)
(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary),seconds=.5,cpu=42),indent=2))
for name in ['neos-5052403-cygnet.mps.gz','sp97ar.mps.gz','tbfp-network.mps.gz']:
 d=out/name;d.mkdir();seed=root/'artifacts/root-objective-repair-v1-paired-41'/name/'before/result.json';model=root/'data/raw/benchmark'/name;cmd=['taskset','-c','42','timeout','--signal=KILL','65',str(out/'bin/primal_descent'),str(model),str(seed),'0.5',str(d/'solution.json')];(d/'input.json').write_text(json.dumps(dict(model=str(model),model_sha256=sha256(model),seed=str(seed),seed_sha256=sha256(seed)),indent=2))
 with(d/'stdout.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:r=subprocess.run(cmd,stdout=stdout,stderr=stderr)
 assert r.returncode==0;a=json.load(open(d/'stdout.json'));assert a['source_fingerprint']==info['source_fingerprint'];records.append(dict(name=name,**a));(out/'summary.json').write_text(json.dumps(records,indent=2))
print(records)
