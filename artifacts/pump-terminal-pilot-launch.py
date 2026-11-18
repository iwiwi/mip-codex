from pathlib import Path
import json,subprocess,sys,shutil,os
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256,atomic_json
out=root/'artifacts/pump-terminal-pilot';out.mkdir();binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);frozen=out/'frozen';frozen.mkdir()
for name in ['src','scripts','tests']:shutil.copytree(root/name,frozen/name,ignore=shutil.ignore_patterns('__pycache__'))
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,frozen/name)
(frozen/'bin').mkdir();shutil.copy2(binary,frozen/'bin/mip-codex');atomic_json(frozen/'SHA256.json',{str(p.relative_to(frozen)):sha256(p)for p in frozen.rglob('*')if p.is_file()});atomic_json(out/'origin.json',dict(git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha256(frozen/'bin/mip-codex')))
source=Path('/home/ubuntu/mip-pump-limited-rounding/artifacts/pump-limited-development-analysis.json');old=json.load(open(source));selected={r['name']for r in old['feature_used']};assert len(selected)==17
manifest=[r for r in json.load(open('data/manifest.json'))['instances']if r['split']=='development'];assert len(manifest)==188;jobs=[r for r in manifest if r['name']in selected];assert len(jobs)==17;atomic_json(out/'jobs.json',jobs);atomic_json(out/'experiment.json',dict(models=17,selection='all initial2d after limited_points>0 development models; fixed before new repair outcomes',selection_analysis_sha256=sha256(source),cpus=[44,49],seconds=10,threads=1,memory_bytes=16*1024**3,flags=[0,1],repair_deadline='minimum100ms/remaining1%',repair_work_budget=500000,extra_lp_budget=False));workers=[]
for shard,cpu in enumerate([44,49]):
 with(out/f'worker-{shard}.log').open('w')as stream:p=subprocess.Popen([sys.executable,'artifacts/pump-terminal-pilot-worker.py',str(cpu),str(shard)],stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(shard=shard,cpu=cpu,pid=p.pid))
atomic_json(out/'workers.json',workers);print(json.dumps(workers),flush=True)
