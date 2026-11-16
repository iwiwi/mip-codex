from pathlib import Path
import hashlib,json
root=Path('artifacts');out=root/'probe-root-warm-final-extra-evidence';assert not out.exists();out.mkdir();ledger=[];excluded=[]
sources=[root/n for n in ['probe-root-warm-gate-long-paired-3','probe-root-warm-gate-long3.log','probe-root-warm-gate-long-paired.py','probe-root-warm-neos60-audit','probe-root-warm-neos60-audit.log','probe-root-warm-neos60-audit.py','probe-root-warm-format-evidence','probe-root-warm-formatted-unit.log','probe-root-warm-formatted-build.log','probe-root-warm-fmt.log','probe-root-warm-fmt-checked.log','probe-root-warm-formatted-snapshot.log','runs/probe-root-warm-formatted-final-20261116']]
for source in sources:
 assert source.exists(),source
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b)));continue
  q=out/source.name/p.relative_to(source)if source.is_dir()else out/source.name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q),sha256=h,bytes=len(b)))
for n in ['src/solve.rs','src/revised.rs','Cargo.toml','Cargo.lock','build.rs','experiments/probe-root-warm-snapshot-20261116.json']:
 p=Path(n);b=p.read_bytes();q=out/'source'/n;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q),sha256=hashlib.sha256(b).hexdigest(),bytes=len(b)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-binaries.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_binaries=len(excluded));(out/'summary.json').write_text(json.dumps(s,indent=2)+'\n');print(json.dumps(s))
