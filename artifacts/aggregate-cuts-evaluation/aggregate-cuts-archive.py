import hashlib,json,shutil
from pathlib import Path
root=Path('artifacts');dest=root/'aggregate-cuts-evaluation';dest.mkdir(exist_ok=True);ledger=[]
def copy(source,target):
 if source.is_dir():shutil.copytree(source,target,dirs_exist_ok=True)
 else:target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=target/original.relative_to(source)if source.is_dir()else target
  digest=hashlib.sha256(original.read_bytes()).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest
  ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=original.stat().st_size))
for source in sorted(root.glob('aggregate-cut*')):
 if source==dest or source.name in ['aggregate-cuts-archive.log','aggregate-cuts-archive.exit']:continue
 copy(source,dest/source.name)
labels=['initial-before','initial-after','before8','after8','before16','after16','long-before','long-after','native-timtab1','native-fhnw-binpack4-4','final-repeat']
runs={Path(Path(f'artifacts/aggregate-cuts-{label}.log').read_text().strip().splitlines()[-1])for label in labels}
for run in sorted(runs):
 assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
copy(root/'runs/baseline-v2-20261115/cases/021-highs',dest/'native-full-new-error/021-highs')
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(json.dumps(dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),smoke_runs=len(runs))))
