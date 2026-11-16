from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-ray-review');out=repo/'artifacts/row-ray-bound-evaluation';assert not out.exists();
for n,points in [('row-ray-bound-development-audit-retry',120),('row-ray-bound-v2-development-audit',122)]:
 a=json.load(open(art/n/'summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==points
for n,count,points in [('row-ray-bound-selected-audit',16,12),('row-ray-bound-final-selected-audit',4,4)]:
 a=json.load(open(art/n/'summary.json'));assert a['records']==a['cut_valid']==count and a['points']==a['point_valid']==points
out.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained; source and build receipt archived'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['row-ray-bound*','cover-dual-packing*','cover-row-packing*']:
 for s in sorted(art.glob(pattern)):copy(s,out/s.name)
copy(Path('/home/ubuntu/mip-dual-restoration/artifacts/smoke-20261116-154157-dgb8t7dl'),out/'native-60-cygnet')
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
for name in ['ROW_RAY_BOUND_RESULTS','ROW_RAY_BOUND_HYPOTHESIS']:copy(repo/'docs'/f'{name}.md',out/'docs'/f'{name}.md')
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
