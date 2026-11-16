from pathlib import Path
import json,hashlib
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/row-packing-bound-evaluation';assert not out.exists()
a=json.load(open(art/'row-packing-bound-v2-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==120
a=json.load(open(art/'row-packing-bound-selected-audit/summary.json'));assert a['records']==a['cut_valid']==20 and a['points']==a['point_valid']==18
for n,count in [('row-packing-bound-native-audit',8),('row-packing-bound-native-60-audit',3)]:
 a=json.load(open(art/n/'summary.json'));assert a['checked']==a['strict_valid']==a['official_valid']==count
out.mkdir(parents=True);ledger=[];excluded=[]
def copy(s,t):
 for p in sorted(s.rglob('*'))if s.is_dir()else[s]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if b[:4]==b'\x7fELF' or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'generated GLPK input model; original MPS SHA, driver source and logs retained'));continue
  q=t/p.relative_to(s)if s.is_dir()else t;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['row-packing-bound*','objective-cut*']:
 for s in sorted(art.glob(pattern)):copy(s,out/s.name)
for n in ['smoke-20261116-161837-5161erw3','smoke-20261116-161836-ns2fyw_s']:copy(repo/'artifacts'/n,out/n)
for n in ['ROW_PACKING_BOUND_RESULTS','ROW_PACKING_BOUND_HYPOTHESIS']:copy(repo/'docs'/f'{n}.md',out/'docs'/f'{n}.md')
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
refs=[]
for p in sorted((art/'component-primal-cygnet-120/components').glob('component-*/model.json')):refs.append(dict(source=str(p),sha256=hashlib.sha256(p.read_bytes()).hexdigest(),bytes=p.stat().st_size,archived_in='artifacts/component-primal-research-evaluation'))
(out/'component-input-references.json').write_text(json.dumps(refs,indent=2)+'\n');(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(x['bytes']for x in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
