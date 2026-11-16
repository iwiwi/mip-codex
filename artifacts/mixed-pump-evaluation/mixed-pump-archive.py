from pathlib import Path
import hashlib,json,subprocess
root=Path('artifacts');out=root/'mixed-pump-evaluation';assert not out.exists();out.mkdir();ledger=[]
for source in sorted(root.glob('mixed-pump-*')):
 if source==out:continue
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();assert b[:4]!=b'\x7fELF';target=out/source.name/p.relative_to(source)if source.is_dir()else out/source.name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);h=hashlib.sha256(b).hexdigest();assert hashlib.sha256(target.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(target),bytes=len(b),sha256=h))
worktree=Path('/home/ubuntu/mip-mixed-pump')
for name in ['src/solve.rs','src/revised.rs','Cargo.toml','Cargo.lock','build.rs']:
 p=worktree/name;b=p.read_bytes();target=out/'source'/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);ledger.append(dict(source=str(p),archived=str(target),bytes=len(b),sha256=hashlib.sha256(b).hexdigest()))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger));(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
