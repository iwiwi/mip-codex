from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair');out=repo/'artifacts/safe-gmi-postmerge-evaluation';assert not out.exists();out.mkdir();ledger=[];excluded=[]
a=json.load(open(root/'artifacts/safe-gmi-primary-audit/summary.json'));assert a['checked']==a['strict_valid']==a['official_valid']==3 and a['cut_runs']==a['cut_valid']==3
paths=[root/'artifacts'/name for name in ['safe-gmi-primary-audit','smoke-20261117-040937-0imh58xs','safe-gmi-pr-ai-usage.json','safe-gmi-github-plugin-create-failure.json','pure-cover-exchange-receipt-relocation-before-pr105.json']]
paths.extend(p for p in (root/'artifacts').iterdir()if p.name.startswith(('safe-gmi-primary-','safe-gmi-postmerge-','safe-gmi-snapshot-'))and p.name!='safe-gmi-postmerge-archive.log'and p not in paths)
run=root/'artifacts/runs/safe-gmi-final-20261117';paths.extend([run/'config.json',run/'snapshot'])
for source in paths:
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  data=p.read_bytes();h=hashlib.sha256(data).hexdigest()
  if data[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(data),reason='local executable/generated model retained with source and SHA'));continue
  relative=Path('snapshot')/p.relative_to(source)if source.name=='snapshot'else p.relative_to(root/'artifacts');q=out/relative;q.parent.mkdir(parents=True,exist_ok=True);assert not q.exists(),q;q.write_bytes(data);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(data)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
