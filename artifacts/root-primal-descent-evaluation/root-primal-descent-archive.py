from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/root-primal-descent-evaluation';assert not out.exists();a=json.load(open(art/'root-primal-descent-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==120
for n,c in [('root-primal-descent-selected-audit',12),('root-primal-descent-default-on-audit',4)]:
 a=json.load(open(art/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
out.mkdir(parents=True);ledger=[];excluded=[]
for s in sorted(art.glob('root-primal-descent*')):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='ELF retained locally with source/build receipt'));continue
  q=out/s.name/p.relative_to(s)if s.is_dir()else out/s.name;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
refs=[]
for p in sorted((art/'root-objective-repair-oracle').glob('*/model.mps')):refs.append(dict(source=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),archived_in='artifacts/root-objective-repair-evaluation/root-objective-repair-oracle'))
(out/'oracle-input-references.json').write_text(json.dumps(refs,indent=2)+'\n');(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
