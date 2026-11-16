from pathlib import Path
import json,hashlib
r=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-objective-repair');src=Path('/home/ubuntu/mip-dual-restoration');a=json.load(open(r/'artifacts/dual-delayed-bland-v1-audit/summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==10
o=repo/'artifacts/dual-delayed-bland-research-evaluation';o.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local frozen ELF retained; exact source/build info archived'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for p in sorted((r/'artifacts').glob('dual-delayed-bland-*')):copy(p,o/p.name)
copy(src/'docs/DUAL_DELAYED_BLAND_HYPOTHESIS.md',o/'docs/DUAL_DELAYED_BLAND_HYPOTHESIS.md');copy(src/'docs/DUAL_DELAYED_BLAND_HYPOTHESIS.md',repo/'docs/DUAL_DELAYED_BLAND_RESULTS.md')
(o/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(o/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');s=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(o/'archive-summary.json').write_text(json.dumps(s,indent=2)+'\n');print(s)
