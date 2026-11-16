from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');art=root/'artifacts';repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/component-primal-research-evaluation';assert not out.exists();out.mkdir(parents=True);ledger=[];excluded=[]
def copy(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  b=p.read_bytes();h=hashlib.sha256(b).hexdigest()
  if p.name=='model.glp' or p.name.startswith('perf.data') or b[:4]==b'\x7fELF':excluded.append(dict(source=str(p),sha256=h,bytes=len(b),reason='local ELF retained with source/build receipt'if b[:4]==b'\x7fELF'else'local sampling stack data; aggregate symbol report retained'if p.name.startswith('perf.data')else'generated GLPK model; input SHA and execution logs retained'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(b);assert hashlib.sha256(q.read_bytes()).hexdigest()==h;ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(b)))
for pattern in ['component-primal*','component-repair*','component-phase-one*','whole-model-repair*']:
 for source in sorted(art.glob(pattern)):copy(source,out/source.name)
copy(art/'phase-one-keep-pricing-final-diagnostic',out/'supporting-phase-one-source')
copy(art/'smoke-20261116-124754-4p4npm9c',out/'primary-postmerge-smoke')
copy(art/'terminal-neighborhood-primary-audit',out/'primary-postmerge-audit')
for name in ['terminal-neighborhood-primary-build.log','terminal-neighborhood-primary-controls.log','terminal-neighborhood-primary-audit.log','terminal-neighborhood-final-snapshot-create.log']:copy(art/name,out/'primary-postmerge-logs'/name)
copy(art/'runs/terminal-neighborhood-final-20261116',out/'snapshot-create-only')
copy(root/'experiments/terminal-neighborhood-snapshot-20261116.json',out/'snapshot-receipt.json')
for n in ['component_primal.rs','repair_component.rs']:copy(repo/'examples'/n,out/'latest-diagnostic-source'/n)
for n in ['COMPONENT_PRIMAL_HYPOTHESIS','ROOT_SEED_REPAIR_HYPOTHESIS']:copy(repo/'docs'/(n+'.md'),out/'latest-diagnostic-source'/(n+'.md'))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
