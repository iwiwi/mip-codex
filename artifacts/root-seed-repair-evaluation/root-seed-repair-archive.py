from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/root-seed-repair-evaluation';assert not out.exists();out.mkdir(parents=True);ledger=[];excluded=[]
a=json.load(open(art/'root-seed-repair-development-audit/summary.json'));assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==116
assert json.load(open(art/'root-seed-repair-final-audit/summary.json'))['point_valid']==4
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if p.name=='model.glp' or p.name.startswith('perf.data') or b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'local sampling stack data; aggregate symbol report retained'if p.name.startswith('perf.data')else'generated GLPK model; input SHA and execution logs retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for source in sorted(art.glob('root-seed-repair*')):copy(source,out/source.name)
for run in ['smoke-20261116-130729-zup4td4q','smoke-20261116-131008-lnq1bzy5','smoke-20261116-132107-mhx8p6qo']:copy(repo/'artifacts'/run,out/'native-smokes'/run)
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
for n in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],cwd=repo,text=True).splitlines():copy(repo/n,out/'final-source'/n)
for n in ['ROOT_SEED_REPAIR_RESULTS','ROOT_SEED_REPAIR_HYPOTHESIS','OBJECTIVE_DOMAIN_REPORT_HYPOTHESIS','INCUMBENT_LP_WARM_HYPOTHESIS','JOURNAL','PLAN']:copy(repo/'docs'/(n+'.md'),out/'final-source/docs'/(n+'.md'))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
