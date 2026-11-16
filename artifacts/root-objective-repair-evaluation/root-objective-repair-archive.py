from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-objective-repair');out=repo/'artifacts/root-objective-repair-evaluation';assert not out.exists()
a=json.load(open(art/'root-objective-repair-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==120
for n,c in [('root-objective-repair-selected-audit',10),('root-objective-repair-default-on-audit',4)]:
 a=json.load(open(art/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
out.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'generated GLPK input; original MPS SHA and driver/logs retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for s in sorted(art.glob('root-objective-repair*')):copy(s,out/s.name)
for n in ['smoke-20261116-163741-g7imgh4y','smoke-20261116-164015-4rfo83vd']:copy(repo/'artifacts'/n,out/n)
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json');copy(repo/'docs/ROOT_OBJECTIVE_REPAIR_HYPOTHESIS.md',out/'docs/ROOT_OBJECTIVE_REPAIR_HYPOTHESIS.md')
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
