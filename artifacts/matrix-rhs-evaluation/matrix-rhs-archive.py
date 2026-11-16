import hashlib,json
from pathlib import Path
root=Path('artifacts');dest=root/'matrix-rhs-evaluation';dest.mkdir(exist_ok=True);ledger=[]
def copy(source,target):
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=target/original.relative_to(source)if source.is_dir()else target;relocated.parent.mkdir(parents=True,exist_ok=True);data=original.read_bytes();relocated.write_bytes(data);digest=hashlib.sha256(data).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest;ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=len(data)))
for source in sorted(root.glob('matrix-rhs-*')):
 if source==dest or source.name.startswith('matrix-rhs-archive')and source.suffix in ['.log','.exit']:continue
 copy(source,dest/source.name)
for label in ['matrix-rhs-own60','matrix-rhs-extra60','matrix-rhs-native-pk1']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
for name in ['src/revised.rs','Cargo.toml','Cargo.lock','build.rs']:copy(Path(name),dest/'source'/name)
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger));(dest/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
