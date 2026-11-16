from pathlib import Path
import json,hashlib,subprocess
root=Path('artifacts');out=root/'probe-root-warm-evaluation';assert not out.exists()
for name in ['probe-root-warm-development-audit','probe-root-warm-gate-development-audit']:
 assert json.loads((root/name/'summary.json').read_text())['completed']==376
primary=Path((root/'probe-root-warm-primary-smoke.log').read_text().strip().splitlines()[-1]);assert(primary/'experiment.json').is_file();assert len((primary/'results.jsonl').read_text().splitlines())==2
out.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();digest=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=digest,bytes=len(b),reason='retained locally; source/build receipts archived'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==digest;ledger.append(dict(source=str(p),archived=str(q),sha256=digest,bytes=len(b)))
for source in sorted(root.glob('probe-root-*')):
 if source==out or source.name=='probe-root-warm-archive.log':continue
 copy(source,out/source.name)
for label in ['probe-root-warm-primary-smoke','mir-root-only-native-nu25']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,out/'smoke-runs'/run.name)
copy(root/'runs/probe-root-warm-final-20261116',out/'full-snapshot-create-only')
copy(root/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
for name in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],text=True).splitlines():copy(Path(name),out/'source'/name)
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded));(out/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(json.dumps(s))
