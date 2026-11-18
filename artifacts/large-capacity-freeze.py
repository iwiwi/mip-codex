from pathlib import Path
import json,shutil,sys,subprocess,os
os.sched_setaffinity(0,{43});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256,atomic_json
binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);out=root/'artifacts/large-capacity-frozen';out.mkdir();
for name in ['src','scripts','tests','examples']:shutil.copytree(root/name,out/name,ignore=shutil.ignore_patterns('__pycache__'))
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,out/name)
(out/'bin').mkdir();shutil.copy2(binary,out/'bin/mip-codex');atomic_json(out/'build-info.json',info);atomic_json(out/'SHA256.json',{str(p.relative_to(out)):sha256(p)for p in sorted(out.rglob('*'))if p.is_file()});atomic_json(root/'artifacts/large-capacity-origin.json',dict(git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha256(out/'bin/mip-codex')))
