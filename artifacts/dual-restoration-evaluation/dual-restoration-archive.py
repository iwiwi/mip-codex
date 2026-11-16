from pathlib import Path
import hashlib,json,subprocess
root=Path('artifacts');out=root/'dual-restoration-evaluation';assert not out.exists()
for n in ['dual-stable-pivot-development-audit','dual-restoration-combined-development-audit']:
 assert json.loads((root/n/'summary.json').read_text())['completed']==376
primary=Path((root/'dual-restoration-primary-smoke.log').read_text().strip().splitlines()[-1]);assert(primary/'experiment.json').is_file();assert json.loads((root/'dual-restoration-primary-audit/summary.json').read_text())['strict_valid']==5
out.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='retained locally with source/build receipts'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q),sha256=h,bytes=len(b)))
for source in sorted(root.glob('dual-*')):
 if source==out or source.name in ['dual-restoration-archive.log','dual-restoration-archive.exit']:continue
 copy(source,out/source.name)
copy(root/'lp-numerical-followup',out/'lp-numerical-followup')
for label in ['dual-restoration-primary-smoke','dual-stable-pivot-native-ran']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,out/'smoke-runs'/run.name)
copy(root/'runs/dual-restoration-final-20261116',out/'full-snapshot-create-only')
copy(root/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
for name in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],text=True).splitlines():copy(Path(name),out/'source'/name)
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
