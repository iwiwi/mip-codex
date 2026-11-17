from pathlib import Path
import json,hashlib,subprocess
root=Path('/home/ubuntu/mip-codex');repo=Path('/home/ubuntu/mip-dual-restoration');out=repo/'artifacts/safe-gmi-final-evaluation';assert not out.exists()
for name,count,points in [('safe-gmi-default-on-short-audit',14,14),('safe-gmi-default-on-long-audit',4,4),('safe-gmi-default-on-control-audit',4,4),('safe-gmi-default-on-native10-audit',2,7)]:
 a=json.loads((root/'artifacts'/name/'summary.json').read_text());assert a['cut_runs']==a['cut_valid']==count and a['checked']==a['strict_valid']==a['official_valid']==points
for name in ['safe-gmi-default-on-paired-42','safe-gmi-default-on-paired-44','safe-gmi-default-on-control-paired-45']:assert json.loads((root/'artifacts'/name/'exit.json').read_text())['status']=='complete'
out.mkdir();ledger=[];excluded=[]
def save(source,target):
 for p in sorted(source.rglob('*'))if source.is_dir()else[source]:
  if not p.is_file():continue
  data=p.read_bytes();h=hashlib.sha256(data).hexdigest()
  if data[:4]==b'\x7fELF'or p.name=='model.glp':excluded.append(dict(source=str(p),sha256=h,bytes=len(data),reason='local ELF/generated GLPK model retained with source and SHA'));continue
  q=target/p.relative_to(source)if source.is_dir()else target;q.parent.mkdir(parents=True,exist_ok=True);q.write_bytes(data);ledger.append(dict(source=str(p),archived=str(q.relative_to(repo)),sha256=h,bytes=len(data)))
for p in sorted((root/'artifacts').glob('safe-gmi-default-on*')):save(p,out/p.name)
for name in ['safe-gmi-work-cap-effect-attribution.json','native-baseline-health-20261117-0400.json','audit_gmi_final_smoke.py','safe-gmi-final-archive.py']:save(root/'artifacts'/name,out/name)
for name in ['SAFE_GOMORY_RESULTS.md','SAFE_GOMORY_HYPOTHESIS.md']:save(repo/'docs'/name,out/'docs'/name)
save(repo/'tests',out/'tests')
patch=subprocess.check_output(['git','diff','main','--','src','tests','scripts/verify_cover_cuts.py'],cwd=repo);p=out/'core-review.patch';p.write_bytes(patch);ledger.append(dict(source='git diff main -- src tests scripts/verify_cover_cuts.py',archived=str(p.relative_to(repo)),sha256=hashlib.sha256(patch).hexdigest(),bytes=len(patch)))
(out/'relocation-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');(out/'retained-local-files.json').write_text(json.dumps(excluded,indent=2)+'\n');summary=dict(files=len(ledger),bytes=sum(r['bytes']for r in ledger),excluded=len(excluded));(out/'archive-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
