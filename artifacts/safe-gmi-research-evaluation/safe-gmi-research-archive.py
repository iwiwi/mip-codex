from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/safe-gmi-research-evaluation';assert not out.exists();out.mkdir();ledger=[];excluded=[]
for name in ['safe-gmi-v6-development-audit','safe-gmi-report-development-audit','safe-gmi-box-development-audit','safe-gmi-work-cap-development-audit']:
 a=json.loads((root/'artifacts'/name/'summary.json').read_text());assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==122
for p in sorted((root/'artifacts').iterdir()):
 if not p.name.startswith(('safe-gomory-','safe-gmi-'))or p.name.startswith('safe-gmi-default-on')or p.name in ['safe-gmi-research-archive.log']:continue
 for source in sorted(p.rglob('*'))if p.is_dir()else[p]:
  if not source.is_file():continue
  content=source.read_bytes();h=hashlib.sha256(content).hexdigest()
  if content[:4]==b'\x7fELF'or source.name=='model.glp':excluded.append(dict(source=str(source),sha256=h,bytes=len(content),reason='local ELF/generated GLPK model; frozen source and SHA retained'));continue
  target=out/source.relative_to(root/'artifacts');target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(content);ledger.append(dict(source=str(source),archived=str(target.relative_to(repo)),sha256=h,bytes=len(content)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
