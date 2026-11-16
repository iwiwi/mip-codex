from pathlib import Path
import json,hashlib,shutil,subprocess
r=Path('/home/ubuntu/mip-objective-repair');o=Path('/home/ubuntu/mip-codex/artifacts/post-primal-subgradient-v1-diagnostic');o.mkdir();files=[*r.glob('src/**/*.rs'),*r.glob('scripts/*.py'),r/'Cargo.toml',r/'Cargo.lock',r/'build.rs'];ledger=[]
for p in files:
 q=o/p.relative_to(r);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);ledger.append(dict(path=str(p.relative_to(r)),sha256=hashlib.sha256(q.read_bytes()).hexdigest()))
q=o/'bin/mip-codex';q.parent.mkdir();shutil.copy2(r/'target/release/mip-codex',q);info=json.loads(subprocess.check_output([str(q.resolve()),'build-info']));(o/'build-info.json').write_text(json.dumps(info,indent=2)+'\n');ledger.append(dict(path='bin/mip-codex',sha256=hashlib.sha256(q.read_bytes()).hexdigest()));(o/'sha256.json').write_text(json.dumps(ledger,indent=2)+'\n');print(info)
