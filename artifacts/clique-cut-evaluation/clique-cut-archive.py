import hashlib,json,shutil
from pathlib import Path
root=Path('artifacts');dest=root/'clique-cut-evaluation';dest.mkdir(exist_ok=True);ledger=[]
def copy(source,target):
 if source.is_dir():shutil.copytree(source,target,dirs_exist_ok=True)
 else:target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=target/original.relative_to(source)if source.is_dir()else target;digest=hashlib.sha256(original.read_bytes()).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest
  ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=original.stat().st_size))
for source in sorted(root.glob('clique-cut-*')):
 if source==dest or source.name in ['clique-cut-archive.log','clique-cut-archive.exit']:continue
 copy(source,dest/source.name)
for label in ['clique-cut-smoke8','clique-cut-smoke10','clique-cut-native-eil33-10','clique-cut-native-eil33-60']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
copy(root/'runs/baseline-v2-20261115/cases/021-glpk',dest/'native-full-error-021-glpk')
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(json.dumps(dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger))))
