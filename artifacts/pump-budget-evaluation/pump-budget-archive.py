from pathlib import Path
import json,hashlib
root=Path('artifacts');out=root/'pump-budget-evaluation';assert not out.exists();out.mkdir();ledger=[];excluded=[]
for source in sorted(root.glob('pump-*')):
 if source==out:continue
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b)));continue
  target=out/source.name/p.relative_to(source)if source.is_dir()else out/source.name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);assert hashlib.sha256(target.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(target),sha256=h,bytes=len(b)))
for name in ['src/solve.rs','src/revised.rs','Cargo.toml','Cargo.lock','build.rs']:
 p=Path('/home/ubuntu/mip-mixed-pump')/name;b=p.read_bytes();target=out/'source'/name;target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(b);ledger.append(dict(source=str(p),archived=str(target),sha256=hashlib.sha256(b).hexdigest(),bytes=len(b)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded));(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
