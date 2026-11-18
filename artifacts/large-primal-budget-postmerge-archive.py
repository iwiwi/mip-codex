from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/large-primal-budget-postmerge-evaluation';assert not out.exists();assert json.load(open('artifacts/large-primal-budget-main-primary-audit/exit.json'))['gmp_valid']==4;roots=[root/p for p in ['artifacts/large-primal-budget-main-update.py', 'artifacts/large-primal-budget-main-update.json', 'artifacts/large-primal-budget-main-update.log', 'artifacts/large-primal-budget-main-build.log', 'artifacts/large-primal-budget-main-smoke.py', 'artifacts/large-primal-budget-main-smoke.log', 'artifacts/large-primal-budget-main-primary-audit', 'artifacts/large-primal-budget-main-untracked-preserved', 'artifacts/large-primal-budget-main-adoption', 'artifacts/large-primal-budget-final-create.py', 'artifacts/large-primal-budget-final-create.log', 'artifacts/large-primal-budget-final-verify.py', 'artifacts/large-primal-budget-final-verify.log', 'artifacts/large-primal-budget-final-snapshot-verification.json', 'artifacts/large-primal-budget-snapshot-cod105-proof.log', 'artifacts/large-primal-budget-snapshot-pairs5000-proof.log', 'artifacts/large-primal-budget-snapshot-depth-cycles-proof.log', 'artifacts/large-primal-budget-postmerge-comment.py', 'artifacts/large-primal-budget-postmerge-comment.md', 'artifacts/large-primal-budget-postmerge-comment.log', 'artifacts/large-primal-budget-postmerge-comment-run.log', 'artifacts/large-primal-budget-postmerge-ai-usage.json', 'artifacts/large-primal-budget-clean-old-caches.py', 'artifacts/large-primal-budget-old-incremental-plan.json', 'artifacts/large-primal-budget-old-cache-symlink-preflight.json', 'artifacts/large-primal-budget-old-cache-cleanup.json', 'artifacts/large-primal-budget-old-cache-cleanup.log', 'artifacts/large-primal-budget-old-cache-frozen-reference-audit.json', 'artifacts/ACTIVE_RESEARCH_LOCAL.json', 'artifacts/runs/large-primal-budget-final-20261118', 'artifacts/runs/cover-choice-heap-final-20261118/config.json', 'artifacts/runs/cover-choice-heap-final-20261118/OBSOLETE.json', 'experiments/large-primal-budget-snapshot-20261118.json', 'artifacts/large-capacity-native721-terminal.json']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
for f in files:
 b=f.read_bytes();r=dict(path=str(f.relative_to(root)),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
 if b[:4]==b'\x7fELF':excluded.append(r);continue
 manifest.append(r)
 if group and size+len(b)>90*1024**2:groups.append(group);group=[];size=0
 group.append(f);size+=len(b)
if group:groups.append(group)
out.mkdir()
for i,group in enumerate(groups,1):
 p=out/f'bundle-{i:03d}.tar.gz'
 with tarfile.open(p,'w:gz',compresslevel=6)as t:
  for f in group:t.add(f,arcname=str(f.relative_to(root)),recursive=False)
 assert p.stat().st_size<95*1024**2
lookup={r['path']:r for r in manifest};seen=set()
for p in sorted(out.glob('bundle-*.tar.gz')):
 with tarfile.open(p)as t:
  for m in t:
   assert m.name not in seen;seen.add(m.name);b=t.extractfile(m).read();r=lookup[m.name];assert len(b)==r['bytes']and hashlib.sha256(b).hexdigest()==r['sha256']
assert seen==set(lookup)
for r in excluded:assert hashlib.sha256((root/r['path']).read_bytes()).hexdigest()==r['sha256']
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,primary_outputs=4,primary_gmp_valid=4,primary_proof_valid=4,snapshot_files=113,libraries=147,snapshot_jobs_launched=0,native_original_complete=720,native_disk_recovery_jobs=1,production_changes=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
