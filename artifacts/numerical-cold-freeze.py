from pathlib import Path
import json,shutil,sys,subprocess,os,re
os.sched_setaffinity(0,{54});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import require_solver_origin,sha256,atomic_json
binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);out=root/'artifacts/numerical-cold-frozen';out.mkdir()
for name in ['src','scripts','tests']:shutil.copytree(root/name,out/name,ignore=shutil.ignore_patterns('__pycache__'))
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,out/name)
(out/'bin').mkdir();shutil.copy2(binary,out/'bin/mip-codex');atomic_json(out/'build-info.json',info);atomic_json(out/'SHA256.json',{str(p.relative_to(out)):sha256(p)for p in sorted(out.rglob('*'))if p.is_file()});origin=dict(git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha256(out/'bin/mip-codex'));atomic_json(root/'artifacts/numerical-cold-origin.json',origin)
log=(root/'artifacts/numerical-cold-tests.log').read_text();matches=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored',log);summary=dict(passed=sum(int(a)for a,b,c in matches),failed=sum(int(b)for a,b,c in matches),ignored=sum(int(c)for a,b,c in matches));assert summary==dict(passed=363,failed=0,ignored=4),summary;atomic_json(root/'artifacts/numerical-cold-test-summary.json',summary)
f=(root/'artifacts/numerical-cold-pilot.log').open('w');p=subprocess.Popen([str(root/'.venv/bin/python'),'artifacts/numerical-cold-pilot.py'],stdout=f,stderr=subprocess.STDOUT,start_new_session=True);atomic_json(root/'artifacts/numerical-cold-pilot-launch.json',dict(pid=p.pid,origin=origin));print(json.dumps(dict(pid=p.pid,origin=origin,tests=summary)))
