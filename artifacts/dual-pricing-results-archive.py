from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');o=r/'artifacts/dual-pricing-negative-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
a=json.load(open(r/'artifacts/dual-devex-joint-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==121
b=json.load(open(r/'artifacts/dual-tie-and-devex-long-audit/summary.json'));assert b['records']==b['cut_valid']==b['points']==b['point_valid']==22
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  with p.open('rb')as f:head=f.read(4)
  h=hashlib.sha256(p.read_bytes()).hexdigest();size=p.stat().st_size
  if head==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=size,reason='local ELF or generated model; source and hashes retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(p.read_bytes());ledger.append(dict(source=str(p),archived=str(q.relative_to(r)),sha256=h,bytes=size))
for pattern in ['dual-devex-*','dual-positive-tie-*','dual-tie-and-devex-*']:
 for p in sorted((r/'artifacts').glob(pattern)):copy(p,o/p.name)
for name,tree in [('devex-source','/home/ubuntu/mip-objective-repair'),('positive-tie-source','/home/ubuntu/mip-priced-repair')]:
 repo=Path(tree)
 for item in ['src','examples/append_lp.rs','examples/reopt_lp.rs','Cargo.toml','Cargo.lock','build.rs']:
  source=repo/item
  if source.exists():copy(source,o/name/item)
for tree in ['/home/ubuntu/mip-objective-repair','/home/ubuntu/mip-priced-repair']:
 for p in (Path(tree)/'docs').glob('DUAL_*HYPOTHESIS.md'):
  if 'DEVEX'in p.name or 'POSITIVE_TIE'in p.name:copy(p,o/'hypotheses'/p.name)
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
