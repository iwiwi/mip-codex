from pathlib import Path
import json,shutil,subprocess,hashlib,os
root=Path.cwd(); prefix='graph-vertex-small-share'; dest=root/'artifacts'/f'{prefix}-frozen';dest.mkdir()
for name in ['src','scripts','tests']:
 shutil.copytree(root/name,dest/name)
for name in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(root/name,dest/name)
(dest/'bin').mkdir();shutil.copy2(root/'target/release/mip-codex',dest/'bin/mip-codex')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
info=json.loads(subprocess.check_output([str(dest/'bin/mip-codex'),'build-info'],text=True))
origin=dict(head=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),build_info=info,binary_sha256=sha(dest/'bin/mip-codex'),default_off=True,vertex_reserved_fraction=0.1)
(dest/'build-info.json').write_text(json.dumps(info,indent=2));(dest/'SHA256.json').write_text(json.dumps({str(p.relative_to(dest)):sha(p) for p in dest.rglob('*') if p.is_file()},indent=2));(root/'artifacts'/f'{prefix}-origin.json').write_text(json.dumps(origin,indent=2))
src=(root/'artifacts/graph-vertex-long-controls.py').read_text()
for suffix,seconds,cpu in [('long',300,52),('short',60,50)]:
 script=root/'artifacts'/f'{prefix}-{suffix}-controls.py'
 text=src.replace('graph-vertex-long-controls',f'{prefix}-{suffix}-controls').replace('graph-vertex-frozen',f'{prefix}-frozen').replace('graph-vertex-origin',f'{prefix}-origin').replace('{47}',f'{{{cpu}}}').replace('cpu=47',f'cpu={cpu}').replace(',300,',f',{seconds},')
 script.write_text(text)
 log=(root/'artifacts'/f'{prefix}-{suffix}-controls.log').open('w')
 p=subprocess.Popen([str(root/'.venv/bin/python'),str(script)],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 (root/'artifacts'/f'{prefix}-{suffix}-launch.json').write_text(json.dumps(dict(pid=p.pid,cpu=cpu,seconds=seconds,origin=origin),indent=2))
 print(suffix,p.pid)
print(json.dumps(origin))
