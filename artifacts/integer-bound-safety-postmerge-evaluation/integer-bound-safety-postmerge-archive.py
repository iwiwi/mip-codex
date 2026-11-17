from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-resource-improve');out=repo/'artifacts/integer-bound-safety-postmerge-evaluation';assert not out.exists();out.mkdir(parents=True);ledger=[];excluded=[]
a=json.load(open(root/'artifacts/integer-bound-safety-final-primary-audit/summary.json'));assert a['checked']==a['strict_valid']==a['official_valid']==0 and a['cut_runs']==a['cut_valid']==2
paths=[root/'artifacts/integer-bound-safety-final-primary-audit',root/'artifacts/smoke-20261117-153137-_vqb30uv']
paths.extend(p for p in (root/'artifacts').iterdir()if p.name.startswith(('integer-bound-safety-final-primary-','integer-bound-safety-final-postmerge-','integer-bound-safety-final-pr-','integer-bound-safety-final-snapshot-','integer-bound-safety-postmerge-'))and p not in paths)
run=root/'artifacts/runs/integer-bound-safety-final-20261117';paths.extend([run/'config.json',run/'snapshot'])
for source in paths:
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  data=p.read_bytes();h=hashlib.sha256(data).hexdigest()
  if data[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(data),reason='local executable/generated model retained with source and SHA'));continue
  relative=Path('snapshot')/p.relative_to(source)if source.name=='snapshot'else p.relative_to(root/'artifacts');q=out/relative;q.parent.mkdir(parents=True,exist_ok=True);assert not q.exists(),q;q.write_bytes(data);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(data)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)

for row in ledger:assert hashlib.sha256((repo/row['archived']).read_bytes()).hexdigest()==row['sha256']
