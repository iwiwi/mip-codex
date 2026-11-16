import hashlib,json,shutil
from pathlib import Path
root=Path('artifacts');dest=root/'pricing-evaluation';dest.mkdir(exist_ok=True);ledger=[];seen=set()
def copy(source,target):
 if source in seen:return
 seen.add(source)
 if source.is_dir():shutil.copytree(source,target,dirs_exist_ok=True)
 else:target.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(source,target)
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  relocated=target/original.relative_to(source)if source.is_dir()else target;digest=hashlib.sha256(original.read_bytes()).hexdigest();assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest;assert original.stat().st_size<95*1024**2
  ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=original.stat().st_size))
for pattern in ['static-pricing-*','feasibility-pricing-*','pricing-*-root-survey']:
 for source in sorted(root.glob(pattern)):
  if source.name.endswith('-bin')or source.name in ['static-pricing-archive.log','static-pricing-archive.exit']:continue
  copy(source,dest/source.name)
for prefix in ['static-pricing-l2','static-pricing-l1','static-pricing-guarded','static-pricing-restart','static-pricing-final','feasibility-pricing','feasibility-pricing-isolated']:
 for label in ['smoke8','smoke10']:
  run=Path((root/f'{prefix}-{label}.log').read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
run=Path((root/'feasibility-pricing-native-krka60.log').read_text().strip().splitlines()[-1]);copy(run,dest/'smoke-runs'/run.name)
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(json.dumps(dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger))))
