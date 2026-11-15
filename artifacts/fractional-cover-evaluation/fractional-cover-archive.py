import hashlib,json,shutil
from pathlib import Path
root=Path('artifacts');dest=root/'fractional-cover-evaluation';dest.mkdir(exist_ok=True)
paths=sorted(p for p in root.glob('fractional-cover-*')if p!=dest and p.name not in ['fractional-cover-archive.log','fractional-cover-archive.exit'])
ledger=[]
for source in paths:
 target=dest/source.name
 if source.is_dir():shutil.copytree(source,target,dirs_exist_ok=True)
 else:shutil.copy2(source,target)
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=dest/original.relative_to(root);digest=hashlib.sha256(original.read_bytes()).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest
  ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=original.stat().st_size))
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(json.dumps(dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger))))
