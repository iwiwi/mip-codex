from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');o=repo/'artifacts/row-subgradient-budget-negative-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
for n,c in [('row-subgradient-budget-v1-audit',14),('row-subgradient-budget-v1-long-audit',8),('row-subgradient-budget-v1-eil-long-audit',2)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
a=json.load(open(r/'artifacts/row-subgradient-budget-native60-audit/summary.json'));assert a['checked']==a['strict_valid']==a['official_valid']==8 and a['cut_valid']==2
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated model; frozen source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('row-subgradient-budget-*')):
 if p.name!='row-subgradient-budget-negative-archive.log':copy(p,o/p.name)
copy(repo/'artifacts/smoke-20261116-212112-e2lurktl',o/'native60');copy(repo/'docs/ROW_SUBGRADIENT_BUDGET_RESULTS.md',o/'docs/ROW_SUBGRADIENT_BUDGET_RESULTS.md')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
