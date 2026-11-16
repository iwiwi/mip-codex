from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/vec-backed-rhs-evaluation';assert not out.exists();out.mkdir(parents=True);ledger=[];excluded=[]
a=json.load(open(art/'vec-backed-rhs-development-audit/summary.json'));assert a['completed']==376 and a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==112
assert json.load(open(art/'vec-backed-rhs-primary-audit/summary.json'))['strict_valid']==11
assert json.load(open(art/'vec-backed-rhs-long-audit/summary.json'))['point_valid']==18

def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if p.name=='model.glp' or p.name.startswith('perf.data') or b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'local sampling stack data; aggregate symbol report retained'if p.name.startswith('perf.data')else'generated GLPK model; input SHA and execution logs retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for source in sorted(art.glob('vec-backed-rhs*')):
 if source.name=='vec-backed-rhs-archive.log':continue
 copy(source,out/source.name)
for pattern in ['dual-denominator-refactor*','profile-main-binkar*']:
 for source in sorted(art.glob(pattern)):copy(source,out/'supporting-records'/source.name)
for label in ['vec-backed-rhs-primary-native10-smoke','vec-backed-rhs-primary-controls']:
 run=art/Path((art/(label+'.log')).read_text().strip().splitlines()[-1]).name;copy(run,out/'smoke-runs'/run.name)
copy(art/'runs/vec-backed-rhs-final-20261116',out/'snapshot-create-only');copy(root/'experiments/vec-backed-rhs-snapshot-20261116.json',out/'snapshot-receipt.json');copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json')
for n in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],cwd=root,text=True).splitlines():copy(root/n,out/'source'/n)
for n in ['docs/VEC_BACKED_RHS_HYPOTHESIS.md','docs/JOURNAL.md','docs/PLAN.md']:copy(root/n,out/'source'/n)
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_elf=sum(r['reason'].startswith('local ELF')for r in excluded),sampling_data_omitted=sum(r['reason'].startswith('local sampling')for r in excluded),generated_models_omitted=sum(r['reason'].startswith('generated GLPK')for r in excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
