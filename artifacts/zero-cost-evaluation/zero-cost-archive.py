import hashlib,json,subprocess
from pathlib import Path
root=Path('artifacts');dest=root/'zero-cost-evaluation';worktree=Path('/home/ubuntu/mip-zero-cost-pricing')
for i in range(5):assert json.loads((root/'zero-cost-probe-development-screen'/f'worker-{i}-exit.json').read_text())['status']=='complete'
assert json.loads((root/'zero-cost-probe-development-audit'/'summary.json').read_text())['completed']==376
assert not dest.exists(),'create-only archive'
dest.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for original in sorted(source.rglob('*')) if source.is_dir() else [source]:
  if not original.is_file():continue
  data=original.read_bytes();digest=hashlib.sha256(data).hexdigest()
  if data[:4]==b'\x7fELF':
   excluded.append(dict(source=str(original),sha256=digest,bytes=len(data),reason='binary retained locally; source and build receipts archived'));continue
  relocated=target/original.relative_to(source) if source.is_dir() else target
  relocated.parent.mkdir(parents=True,exist_ok=True);relocated.write_bytes(data)
  assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest
  ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=len(data)))
for source in sorted(root.glob('zero-cost-*')):
 if source==dest:continue
 copy(source,dest/source.name)
for source in sorted(root.glob('zero-objective-pricing*')):copy(source,dest/source.name)
run=Path((root/'zero-cost-native-krka.log').read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
copy(root/'runs/zero-cost-pricing-final-20261116',dest/'full-snapshot-create-only')
copy(root/'runs/baseline-v2-20261115/cases/045-glpk',dest/'native-full-errors/045-glpk')
copy(root/'runs/baseline-v2-20261115/progress.json',dest/'native-full-progress.json')
for name in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],cwd=worktree,text=True).splitlines():copy(worktree/name,dest/'source'/name)
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(dest/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n')
summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded))
(dest/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
