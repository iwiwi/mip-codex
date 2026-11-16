from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair');a=json.load(open(r/'artifacts/priced-row-subgradient-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']
for n,c in [('priced-row-subgradient-v1-audit',10),('priced-row-subgradient-v1-long-audit',4),('priced-row-subgradient-v1-swath-audit',4),('priced-row-subgradient-default-on-audit',14),('priced-row-subgradient-default-on-long-audit',4)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
oracle=json.load(open(r/'artifacts/priced-row-subgradient-oracle/summary.json'));assert oracle['valid']==200 and oracle['new_proofs']==162 and oracle['tampered_rejected']==648
o=repo/'artifacts/priced-row-subgradient-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated GLPK input; original source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['priced-row-subgradient-*','priced-row-reduced-cost-*']:
 for p in sorted((r/'artifacts').glob(pattern)):
  if p.name!='priced-row-subgradient-archive.log':copy(p,o/p.name)
copy(repo/'artifacts/smoke-20261116-194900-bfwzv_84',o/'smoke-20261116-194900-bfwzv_84');copy(repo/'artifacts/smoke-20261116-195209-umwfcsv9',o/'smoke-20261116-195209-umwfcsv9')
copy(repo/'docs/PRICED_ROW_SUBGRADIENT_HYPOTHESIS.md',o/'docs/PRICED_ROW_SUBGRADIENT_HYPOTHESIS.md');copy(repo/'docs/PRICED_ROW_REDUCED_COST_HYPOTHESIS.md',o/'docs/PRICED_ROW_REDUCED_COST_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
