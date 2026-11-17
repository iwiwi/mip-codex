from pathlib import Path
import hashlib,json,subprocess,shutil
r=Path('/home/ubuntu/mip-priced-repair');o=Path('/home/ubuntu/mip-codex/artifacts/dual-refactor-retry-default-on-helpers');o.mkdir();(o/'bin').mkdir();info=json.loads(subprocess.check_output([str(r/'target/release/mip-codex'),'build-info']));ledger=[]
for name in ['append_lp','reopt_lp']:
 source=r/'examples'/(name+'.rs');shutil.copy2(source,o/source.name);binary=r/'target/release/examples'/name;destination=o/'bin'/name;shutil.copy2(binary,destination);origin=json.loads(subprocess.check_output([str(destination),'build-info']));assert origin['library_source_fingerprint']==info['source_fingerprint'];ledger.append(dict(name=name,path=str(destination),sha256=hashlib.sha256(destination.read_bytes()).hexdigest(),source_sha256=hashlib.sha256(source.read_bytes()).hexdigest(),build_info=origin))
(o/'experiment.json').write_text(json.dumps(dict(solver_build_info=info,helpers=ledger),indent=2)+'\n');print(info)
