from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');a=json.load(open(r/'artifacts/priced-objective-root-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']
for n,c in [('priced-objective-root-v1-audit',8),('priced-objective-root-v1-long-audit',4),('priced-objective-root-default-on-audit',10),('priced-objective-root-default-on-long-audit',4)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
o=repo/'artifacts/priced-objective-root-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated GLPK input; original source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['priced-objective-root-*','priced-objective-cut-*']:
 for p in sorted((r/'artifacts').glob(pattern)):copy(p,o/p.name)
copy(repo/'artifacts/smoke-20261116-182413-68pn7iai',o/'smoke-20261116-182413-68pn7iai')
copy(repo/'docs/PRICED_OBJECTIVE_CUT_HYPOTHESIS.md',o/'docs/PRICED_OBJECTIVE_CUT_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
