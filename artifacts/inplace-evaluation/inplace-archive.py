import hashlib,json,shutil
from pathlib import Path
root=Path('artifacts');dest=root/'inplace-evaluation';dest.mkdir(exist_ok=True);ledger=[]
def copy(source,target):
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=target/original.relative_to(source)if source.is_dir()else target;relocated.parent.mkdir(parents=True,exist_ok=True);data=original.read_bytes();relocated.write_bytes(data);digest=hashlib.sha256(data).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest;ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=len(data)))
for source in sorted(root.glob('inplace-*')):
 if source==dest or source.name in ['inplace-archive.log','inplace-archive.exit']:continue
 copy(source,dest/source.name)
for label in ['inplace-own60','inplace-extra60','four-rounds-native-binkar60','four-rounds-native-timtab60','root-cut-native-neos60']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
for name in ['030-glpk','031-glpk','076-glpk']:
 copy(root/'runs/baseline-v2-20261115/cases'/name,dest/'native-errors'/name)
for name in ['src/revised.rs','Cargo.toml','Cargo.lock','build.rs']:
 copy(Path(name),dest/'source'/name)
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(json.dumps(dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger))))
