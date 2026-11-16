from pathlib import Path
import json,hashlib,subprocess
root=Path('artifacts');out=root/'decimal-row-mir-evaluation';out.mkdir(exist_ok=False);assert json.load(open(root/'decimal-row-mir-development-audit/summary.json'))['completed']==376;ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b)));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q),sha256=h,bytes=len(b)))
for source in sorted(root.glob('decimal-row-mir*')):
 if source==out or source.name in ['decimal-row-mir-archive.log']:continue
 copy(source,out/source.name)
w=Path('/home/ubuntu/mip-dual-refinement')
for name in subprocess.check_output(['git','ls-files','src','scripts','tests','examples','Cargo.toml','Cargo.lock','build.rs'],cwd=w,text=True).splitlines():copy(w/name,out/'source'/name)
copy(w/'docs/DECIMAL_ROW_MIR_HYPOTHESIS.md',out/'source/docs/DECIMAL_ROW_MIR_HYPOTHESIS.md');(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded),result='not adopted: zero primal improvements and one primal regression');(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
