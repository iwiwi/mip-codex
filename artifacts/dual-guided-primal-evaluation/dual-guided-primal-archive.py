from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');a=json.load(open(r/'artifacts/dual-guided-primal-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==121
for n,c,p in [('dual-guided-primal-v1-audit',10,10),('dual-guided-primal-v1-long-audit',4,4),('dual-guided-primal-v1-sp97-long-audit',2,2),('dual-guided-primal-default-on-audit',14,12),('dual-guided-primal-default-on-long-audit',6,6),('dual-guided-primal-default-on-sp98-long-audit',2,2)]:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['cut_valid']==c and a['points']==a['point_valid']==p
for n in ['dual-guided-repair-diagnostic-audit','dual-guided-portfolio-diagnostic-audit','dual-guided-portfolio-short-diagnostic-audit']:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['records']==a['valid']==10
for n in ['dual-guided-primal-native10-audit','dual-guided-primal-native60-audit']:
 a=json.load(open(r/'artifacts'/n/'summary.json'));assert a['checked']==a['strict_valid']==a['official_valid']==8 and a['cut_valid']==2
o=repo/'artifacts/dual-guided-primal-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local frozen ELF or generated GLPK input; exact source and original MPS hashes retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['dual-guided-primal-*','dual-guided-repair-*','dual-guided-portfolio-*','unit-cover-subgradient-*']:
 for p in sorted((r/'artifacts').glob(pattern)):
  if p.name!='dual-guided-primal-archive.log':copy(p,o/p.name)
for n in ['smoke-20261116-202436-uaqf3jbo','smoke-20261116-202619-ir0iao_0']:copy(repo/'artifacts'/n,o/n)
copy(repo/'docs/DUAL_GUIDED_PRIMAL_HYPOTHESIS.md',o/'docs/DUAL_GUIDED_PRIMAL_HYPOTHESIS.md');copy(r/'artifacts/runs/baseline-v2-20261115/progress.json',o/'native-full-progress.json')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
