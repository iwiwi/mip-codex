from pathlib import Path
import json,hashlib,shutil,subprocess
r=Path('/home/ubuntu/mip-priced-repair');o=Path('/home/ubuntu/mip-codex/artifacts/uniform-row-subgradient-default-on-diagnostic');o.mkdir();files=[*r.glob('src/**/*.rs'),*r.glob('scripts/*.py'),r/'Cargo.toml',r/'Cargo.lock',r/'build.rs'];ledger=[]
for p in files:
 q=o/p.relative_to(r);q.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(p,q);ledger.append(dict(path=str(p.relative_to(r)),sha256=hashlib.sha256(q.read_bytes()).hexdigest()))
q=o/'bin/mip-codex';q.parent.mkdir();shutil.copy2(r/'target/release/mip-codex',q);info=json.loads(subprocess.check_output([str(q.resolve()),'build-info']));(o/'build-info.json').write_text(json.dumps(info,indent=2)+'\n');ledger.append(dict(path='bin/mip-codex',sha256=hashlib.sha256(q.read_bytes()).hexdigest()));(o/'sha256.json').write_text(json.dumps(ledger,indent=2)+'\n');print(info)
b=Path('/home/ubuntu/mip-codex/artifacts/uniform-row-subgradient-final-development-screen');b.mkdir();record=dict(path=str(q),sha256=ledger[-1]['sha256'],build_info=info);(b/'experiment.json').write_text(json.dumps(dict(shards=5,seconds=10,threads=1,memory_bytes=17179869184,binaries={v:record for v in ['before','after']}),indent=2)+'\n')
