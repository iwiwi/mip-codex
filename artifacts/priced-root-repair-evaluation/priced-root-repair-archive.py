from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-priced-repair');out=repo/'artifacts/priced-root-repair-evaluation';assert not out.exists();a=json.load(open(art/'priced-root-repair-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==120
for n,c in [('priced-root-repair-selected-audit',8),('priced-root-repair-long-audit',6),('priced-root-repair-default-on-audit',8)]:
 a=json.load(open(art/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
out.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'generated GLPK input; original MPS SHA/driver/logs retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['priced-root-repair*','priced-primal-repair*']:
 for s in sorted(art.glob(pattern)):copy(s,out/s.name)
for n in ['smoke-20261116-173829-_qfhka8j','smoke-20261116-174417-n1epaah2']:copy(repo/'artifacts'/n,out/n)
for n in ['PRICED_ROOT_REPAIR_HYPOTHESIS','PRICED_PRIMAL_REPAIR_HYPOTHESIS']:copy(repo/'docs'/f'{n}.md',out/'docs'/f'{n}.md')
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
