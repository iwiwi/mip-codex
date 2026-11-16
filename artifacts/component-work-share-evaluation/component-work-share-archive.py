from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair')
for n,c in [('component-work-share-v1-long-audit',2),('component-work-share-v1-300-audit',2),('component-work-share-default-on-audit',8),('component-work-share-default-on-long-audit',2),('component-work-share-default-on-300-audit',2),('component-work-share-default-on-controls-audit',10)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
a=json.load(open(r/'artifacts/component-work-share-oracle/summary.json'));assert a['cases']==a['valid']==200 and a['eligible']==100 and a['improved']==96
a=json.load(open(r/'artifacts/component-work-share-native300-audit/summary.json'));assert a['strict_valid']==a['official_valid']==3 and a['cut_valid']==1
o=repo/'artifacts/component-work-share-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local frozen ELF or generated model; source/hash retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('component-work-share-*')):
 if p.name!='component-work-share-archive.log':copy(p,o/p.name)
copy(repo/'artifacts/smoke-20261116-224129-psvgt3pb',o/'native300');copy(repo/'docs/COMPONENT_WORK_SHARE_HYPOTHESIS.md',o/'docs/COMPONENT_WORK_SHARE_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
