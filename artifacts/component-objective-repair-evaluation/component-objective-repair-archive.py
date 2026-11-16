from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair')
for n,c in [('component-objective-repair-v1-audit',8),('component-objective-repair-v1-long-audit',2),('component-objective-repair-v1-controls-audit',10),('component-objective-repair-v1-300-audit',2),('component-objective-repair-default-on-audit',8),('component-objective-repair-default-on-long-audit',2),('component-objective-repair-default-on-controls-audit',10),('component-objective-repair-default-on-300-audit',2)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
a=json.load(open(r/'artifacts/component-objective-repair-oracle/summary.json'));assert a['cases']==a['valid']==200 and a['eligible']==100 and a['improved']==96
a=json.load(open(r/'artifacts/component-objective-point-audit/summary.json'));assert a['records']==a['valid']==4
for n in ['component-objective-repair-native60-audit','component-objective-repair-final-native60-audit']:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['strict_valid']==a['official_valid']==3 and a['cut_valid']==1
o=repo/'artifacts/component-objective-repair-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated GLPK input; original source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['component-objective-repair-*','component-objective-point-*']:
 for p in sorted((r/'artifacts').glob(pattern)):
  if p.name!='component-objective-repair-archive.log':copy(p,o/p.name)
for name in ['smoke-20261116-215201-kxu2cmwq','smoke-20261116-221159-0qx7rwf8']:copy(repo/'artifacts'/name,o/name)
copy(repo/'docs/COMPONENT_OBJECTIVE_REPAIR_HYPOTHESIS.md',o/'docs/COMPONENT_OBJECTIVE_REPAIR_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
