from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');o=repo/'artifacts/unit-objective-budget-negative-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
a=json.load(open(r/'artifacts/unit-objective-budget-point-audit/summary.json'));assert a['records']==a['valid']==4
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated model; frozen source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('unit-objective-budget-*')):
 if p.name!='unit-objective-budget-negative-archive.log':copy(p,o/p.name)
copy(repo/'docs/UNIT_OBJECTIVE_BUDGET_RESULTS.md',o/'docs/UNIT_OBJECTIVE_BUDGET_RESULTS.md')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
