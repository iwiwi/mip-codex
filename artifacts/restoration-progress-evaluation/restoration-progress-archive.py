from pathlib import Path
import json,hashlib,subprocess
root=Path('artifacts');out=root/'restoration-progress-evaluation';assert not out.exists();assert json.load(open(root/'restoration-progress-development-audit/summary.json'))['completed']==376;assert json.load(open(root/'restoration-progress-primary-audit/summary.json'))['strict_valid']==5
out.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  if p.name=='model.glp':excluded.append(dict(source=str(p),reason='native holdout intermediate model not inspected; input SHA and execution logs retained'));continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='retained locally with source/build receipts'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q),sha256=h,bytes=len(b)))
for prefix in ['restoration-progress','restoration-cold-path','phase-one-contradiction','numerical-equilibration']:
 for source in sorted(root.glob(prefix+'*')):
  if source==out or source.name in ['restoration-progress-archive.log','restoration-progress-archive.exit']:continue
  copy(source,out/source.name)
for label in ['restoration-progress-primary-smoke','restoration-progress-primary-binkar','restoration-progress-native-nidda']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,out/'smoke-runs'/run.name)
for label,name in [('current-snapshot-create-only','restoration-progress-final-20261116'),('baseline-snapshot-create-only','dual-restoration-final-20261116')]:copy(root/'runs'/name,out/label)
copy(root/'aggregate-three-grid-diagnostic-20261116/scripts/verify_cover_cuts.py',out/'recorded-independent-checker/verify_cover_cuts.py')
copy(root/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json');copy(root/'runs/baseline-v2-20261115/cases/055-glpk',out/'native-full-new-errors/055-glpk')
for name in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],text=True).splitlines():copy(Path(name),out/'source'/name)
for name in ['docs/RESTORATION_PROGRESS_RESULTS.md','docs/JOURNAL.md','docs/PLAN.md']:copy(Path(name),out/'source'/name)
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=sum('sha256'in r for r in excluded),other_excluded=sum('sha256'not in r for r in excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
