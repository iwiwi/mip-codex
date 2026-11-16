from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');a=json.load(open(r/'artifacts/post-primal-subgradient-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']
a=json.load(open(r/'artifacts/post-primal-subgradient-final-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']
for n,c in [('post-primal-subgradient-v1-audit',14),('post-primal-subgradient-v1-long-audit',10),('post-primal-subgradient-v1-swath-audit-retry',4),('post-primal-subgradient-default-on-audit',20),('post-primal-subgradient-default-on-long-audit',8)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==c
o=repo/'artifacts/post-primal-subgradient-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF or generated GLPK input; original source/hash retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('post-primal-subgradient-*')):
 if p.name!='post-primal-subgradient-archive.log':copy(p,o/p.name)
copy(repo/'artifacts/smoke-20261116-213549-wx61t47u',o/'native10')
copy(repo/'docs/POST_PRIMAL_SUBGRADIENT_HYPOTHESIS.md',o/'docs/POST_PRIMAL_SUBGRADIENT_HYPOTHESIS.md');copy(r/'artifacts/cache-cleanup-20261116.json',o/'cache-cleanup-20261116.json');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
