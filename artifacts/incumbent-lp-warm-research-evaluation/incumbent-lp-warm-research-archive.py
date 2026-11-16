from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/incumbent-lp-warm-research-evaluation';assert not out.exists();out.mkdir(parents=True);ledger=[];excluded=[]
a=json.load(open(art/'incumbent-lp-warm-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==120
a=json.load(open(art/'incumbent-lp-warm-research-audit/summary.json'));assert a['records']==a['cut_valid']==a['points']==a['point_valid']==46
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if p.name=='model.glp' or p.name.startswith('perf.data') or b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'local sampling stack data; aggregate symbol report retained'if p.name.startswith('perf.data')else'generated GLPK model; input SHA and execution logs retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for source in sorted(art.glob('incumbent-lp-warm*')):copy(source,out/source.name)
for n in subprocess.check_output(['git','ls-files','src','scripts','tests','examples','Cargo.toml','Cargo.lock','build.rs'],cwd=repo,text=True).splitlines():copy(repo/n,out/'latest-unadopted-source'/n)
for n in ['INCUMBENT_LP_WARM_RESULTS','INCUMBENT_LP_WARM_HYPOTHESIS']:copy(repo/'docs'/(n+'.md'),out/'latest-unadopted-source/docs'/(n+'.md'))
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
