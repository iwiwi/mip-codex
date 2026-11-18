from pathlib import Path
import json,shutil,sys,subprocess,os
os.sched_setaffinity(0,{53});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256,atomic_json
assert all(subprocess.run(['pgrep','-x',n],capture_output=True).returncode!=0 for n in ['cargo','rustc']);binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);out=root/'artifacts/farkas-row-projection-default-frozen';out.mkdir()
for name in ['src','scripts','tests']:shutil.copytree(root/name,out/name,ignore=shutil.ignore_patterns('__pycache__'))
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,out/name)
(out/'bin').mkdir();shutil.copy2(binary,out/'bin/mip-codex');atomic_json(out/'build-info.json',info);atomic_json(out/'SHA256.json',{str(p.relative_to(out)):sha256(p)for p in sorted(out.rglob('*'))if p.is_file()});origin=dict(git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha256(out/'bin/mip-codex'),default_on=True,parent=json.load(open('artifacts/farkas-row-projection-default-parent.json')));atomic_json(root/'artifacts/farkas-row-projection-default-origin.json',origin)
f=(root/'artifacts/farkas-row-projection-default-launch-screen.log').open('w');subprocess.run([str(root/'.venv/bin/python'),'artifacts/farkas-row-projection-default-launch-screen.py'],stdout=f,stderr=subprocess.STDOUT,check=True)
for name in ['full-tests','analysis-wait']:
 f=(root/'artifacts'/f'farkas-row-projection-default-{name}.log').open('w');p=subprocess.Popen([str(root/'.venv/bin/python'),f'artifacts/farkas-row-projection-default-{name}.py'],stdout=f,stderr=subprocess.STDOUT,start_new_session=True);atomic_json(root/'artifacts'/f'farkas-row-projection-default-{name}-launch.json',dict(pid=p.pid))
print(json.dumps(origin))
