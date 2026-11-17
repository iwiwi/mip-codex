from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-priced-repair');o=repo/'artifacts/cover-greedy-portfolio-evaluation';assert not o.exists();o.mkdir();ledger=[];excluded=[]
a=json.load(open(r/'artifacts/cover-greedy-portfolio-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==120
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated model; frozen source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('cover-greedy-portfolio-*')):
 if p.name!='cover-greedy-portfolio-archive.log':copy(p,o/p.name)
copy(Path('/home/ubuntu/mip-priced-repair/artifacts/smoke-20261117-003303-hi1dfvhc'),o/'native10-v1')
copy(Path('/home/ubuntu/mip-priced-repair/artifacts/smoke-20261117-005029-8cy3yy1o'),o/'native10-final')
copy(repo/'docs/COVER_GREEDY_PORTFOLIO_HYPOTHESIS.md',o/'docs/COVER_GREEDY_PORTFOLIO_HYPOTHESIS.md')
copy(repo/'tests/pure_cover_greedy.rs',o/'tests-final.rs')
initial=(repo/'tests/pure_cover_greedy.rs').read_text().replace('pure_cover_greedy:true,pure_cover_portfolio:false,','pure_cover_greedy:true,',1)
(o/'tests-before-mode-correction.rs').write_text(initial)
ledger.append(dict(source=str(repo/'tests/pure_cover_greedy.rs'),transform='reconstructed prior fixture by omitting explicit single-mode flag',archived=str((o/'tests-before-mode-correction.rs').relative_to(repo)),sha256=hashlib.sha256(initial.encode()).hexdigest(),bytes=len(initial.encode())))
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
