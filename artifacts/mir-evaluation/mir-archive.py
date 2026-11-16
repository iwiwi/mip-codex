import hashlib,json,subprocess
from pathlib import Path
root=Path('artifacts');dest=root/'mir-evaluation';dest.mkdir(exist_ok=True);ledger=[];excluded=[]
def copy(source,target):
 for original in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not original.is_file():continue
  data=original.read_bytes();digest=hashlib.sha256(data).hexdigest()
  if original.name=='mip-codex'and original.parent.name=='bin':
   excluded.append(dict(source=str(original),sha256=digest,bytes=len(data),reason='frozen binary retained locally; source and receipts archived'));continue
  relocated=target/original.relative_to(source)if source.is_dir()else target;relocated.parent.mkdir(parents=True,exist_ok=True);relocated.write_bytes(data);assert hashlib.sha256(relocated.read_bytes()).hexdigest()==digest;ledger.append(dict(source=str(original),archived=str(relocated),sha256=digest,bytes=len(data)))
for source in sorted(root.glob('mir-*')):
 if source==dest or source.name.startswith('mir-archive')and source.suffix in ['.log','.exit']:continue
 copy(source,dest/source.name)
for label in ['mir-smoke8-venv','mir-smoke10-venv','mir-own60','mir-gate-native-timtab10','mir-gate-own60','mir-gate-extra60','mir-root-only-native-nu25']:
 run=Path((root/(label+'.log')).read_text().strip().splitlines()[-1]);assert(run/'experiment.json').is_file();copy(run,dest/'smoke-runs'/run.name)
files=subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],text=True).splitlines()
for name in files:copy(Path(name),dest/'source'/name)
(dest/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(dest/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded));(dest/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
