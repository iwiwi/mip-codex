from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/cover-dual-subgradient-evaluation';out.mkdir(parents=True);ledger=[];excluded=[]
for pattern in ['cover-dual-subgradient*','cover-dual-native-lp-comparison*']:
 for s in sorted(art.glob(pattern)):
  for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
   if not p.is_file():continue
   b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
   if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='unchanged v2 ELF retained locally; v1 source/hash only'));continue
   q=out/s.name/p.relative_to(s)if s.is_dir()else out/s.name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for n in ['examples/cover_dual_subgradient.rs','docs/COVER_DUAL_SUBGRADIENT_RESULTS.md']:
 p=repo/n;q=out/'source'/n;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes())
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
