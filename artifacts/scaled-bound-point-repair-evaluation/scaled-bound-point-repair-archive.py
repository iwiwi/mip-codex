from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair')
a=json.load(open(r/'artifacts/scaled-bound-point-repair-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']
a=json.load(open(r/'artifacts/scaled-bound-point-repair-default-on-audit/summary.json'));assert a['records']==a['cut_valid']==12 and a['points']==a['point_valid']==8
a=json.load(open(r/'artifacts/scaled-bound-point-repair-sp97-long-audit/summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==2
for prefix in ['scaled-bound-point-repair-oracle-point-audit','scaled-bound-point-repair-final-oracle-point-audit']:
 a=json.load(open(r/'artifacts'/prefix/'summary.json'));assert a['records']==a['valid']==500 and a['invalid']==0
for prefix in ['scaled-bound-point-repair-units114-audit','scaled-bound-point-repair-final-units114-audit']:
 for kind in ['strict','official']:assert json.load(open(r/'artifacts'/prefix/(kind+'-validation.json')))['valid']is True
assert json.load(open(r/'artifacts/scaled-bound-point-repair-units114-dual-audit.json'))['reported_bound_supported']is True
o=repo/'artifacts/scaled-bound-point-repair-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local frozen ELF or generated model; source/hash retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('scaled-bound-point-*')):
 if p.name!='scaled-bound-point-repair-archive.log':copy(p,o/p.name)
copy(r/'artifacts/lp-primal-edge-pricing-oracle/units/root-units-failure-114.mps',o/'baseline-failure-input.mps')
copy(repo/'docs/SCALED_BOUND_POINT_REPAIR_HYPOTHESIS.md',o/'docs/SCALED_BOUND_POINT_REPAIR_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
