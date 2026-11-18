from pathlib import Path
import json,subprocess,sys,shutil,time,os
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256,atomic_json
out=root/'artifacts/pump-limited-pilot';out.mkdir();binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);frozen=out/'frozen';frozen.mkdir()
for name in ['src','scripts','tests']:shutil.copytree(root/name,frozen/name,ignore=shutil.ignore_patterns('__pycache__'))
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,frozen/name)
(frozen/'bin').mkdir();shutil.copy2(binary,frozen/'bin/mip-codex');atomic_json(frozen/'SHA256.json',{str(p.relative_to(frozen)):sha256(p)for p in frozen.rglob('*')if p.is_file()});atomic_json(out/'origin.json',dict(git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha256(frozen/'bin/mip-codex')))
manifest=[r for r in json.load(open('data/manifest.json'))['instances']if r['split']=='development'];assert len(manifest)==188;source=Path('/home/ubuntu/mip-graph-large-primal-budget/artifacts/large-primal-budget-default-development-screen/cases');jobs=[]
for row in manifest:
 p=source/row['name']/'before/result.json';r=json.load(open(p))
 if(r.get('pump')or{}).get('limited',0)>0:jobs.append(dict(**row,selection_source_result_sha256=sha256(p),selection_pump=r['pump']))
assert jobs;atomic_json(out/'jobs.json',jobs);atomic_json(out/'experiment.json',dict(models=len(jobs),selection='all development188 baseline602-before cases with pump.limited>0; selected before new outcomes, no holdout',cpus=[40,47],seconds=10,threads=1,memory_bytes=16*1024**3,flags=[0,1],extra_lp_budget=False));workers=[]
for shard,cpu in enumerate([40,47]):
 with(out/f'worker-{shard}.log').open('w')as stream:p=subprocess.Popen([sys.executable,'artifacts/pump-limited-pilot-worker.py',str(cpu),str(shard)],stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(shard=shard,cpu=cpu,pid=p.pid))
atomic_json(out/'workers.json',workers);print(json.dumps(dict(models=len(jobs),workers=workers)),flush=True)
