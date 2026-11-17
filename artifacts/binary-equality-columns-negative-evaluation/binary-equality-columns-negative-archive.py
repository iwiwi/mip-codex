from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair');out=repo/'artifacts/binary-equality-columns-negative-evaluation';assert not out.exists();out.mkdir();ledger=[];excluded=[]
a=json.loads((root/'artifacts/binary-equality-columns-v2-development-audit/summary.json').read_text());assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==122
for p in sorted((root/'artifacts').glob('binary-equality-*')):
 if p.name=='binary-equality-columns-negative-archive.log':continue
 for source in sorted(p.rglob('*'))if p.is_dir()else[p]:
  if not source.is_file():continue
  content=source.read_bytes();h=hashlib.sha256(content).hexdigest()
  if content[:4]==b'\x7fELF'or source.name=='model.glp':excluded.append(dict(source=str(source),sha256=h,bytes=len(content),reason='local executable retained with source and hash'));continue
  target=out/source.relative_to(root/'artifacts');target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(content);ledger.append(dict(source=str(source),archived=str(target.relative_to(repo)),sha256=h,bytes=len(content)))
text=subprocess.check_output(['git','show','binary-equality-columns:docs/BINARY_EQUALITY_COLUMNS_HYPOTHESIS.md'],cwd=Path('/home/ubuntu/mip-objective-repair'));p=out/'hypothesis.md';p.write_bytes(text);ledger.append(dict(source='git binary-equality-columns:docs/BINARY_EQUALITY_COLUMNS_HYPOTHESIS.md',archived=str(p.relative_to(repo)),sha256=hashlib.sha256(text).hexdigest(),bytes=len(text)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
