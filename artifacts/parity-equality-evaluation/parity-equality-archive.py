from pathlib import Path
import json,hashlib,subprocess,time
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/parity-equality-evaluation';assert not out.exists()
while not(art/'parity-equality-development-audit/exit.json').exists():time.sleep(2)
a=json.load(open(art/'parity-equality-development-audit/summary.json'));assert a['completed']==376 and a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid'];assert json.load(open(art/'parity-equality-development-audit/exit.json'))['status']=='complete'
assert json.load(open(art/'parity-equality-long-audit/summary.json'))['point_valid']==13
assert json.load(open(art/'parity-equality-primary-audit/summary.json'))['strict_valid']==5
out.mkdir();ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if p.name=='model.glp' or b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'generated GLPK model; input SHA and execution logs retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for source in sorted(art.glob('parity-equality*')):
 if source.name in ['parity-equality-archive.log']:continue
 copy(source,out/source.name)
for label in ['parity-equality-native10-smoke','parity-equality-native10-retry-smoke','parity-equality-primary-native10-smoke','parity-equality-primary-controls']:
 run=art/Path((art/(label+'.log')).read_text().strip().splitlines()[-1]).name;assert(run/'experiment.json').is_file();copy(run,out/'smoke-runs'/run.name)
copy(art/'runs/parity-equality-final-20261116',out/'snapshot-create-only');copy(root/'experiments/parity-equality-snapshot-20261116.json',out/'snapshot-receipt.json')
copy(art/'runs/baseline-v2-20261115/progress.json',out/'native-full-progress.json');copy(art/'runs/baseline-v2-20261115/cases/077-glpk',out/'native-full-new-errors/077-glpk')
for n in subprocess.check_output(['git','ls-files','src','scripts','tests','Cargo.toml','Cargo.lock','build.rs'],cwd=root,text=True).splitlines():copy(root/n,out/'source'/n)
for n in ['docs/PARITY_EQUALITY_HYPOTHESIS.md','docs/JOURNAL.md','docs/PLAN.md']:copy(root/n,out/'source'/n)
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),retained_local_elf=sum(r['reason'].startswith('local ELF')for r in excluded),generated_models_omitted=sum(not r['reason'].startswith('local ELF')for r in excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
