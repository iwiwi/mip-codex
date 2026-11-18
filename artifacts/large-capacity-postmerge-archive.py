from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/large-capacity-postmerge-evaluation';assert not out.exists();assert json.load(open('artifacts/large-capacity-main-primary-audit/exit.json'))['gmp_valid']==4;roots=[root/'artifacts'/p for p in ['large-capacity-main-update.py', 'large-capacity-main-update.json', 'large-capacity-main-update.log', 'large-capacity-main-build.log', 'large-capacity-main-smoke.py', 'large-capacity-main-smoke.log', 'large-capacity-main-primary-audit', 'large-capacity-main-debug-cleanup.json', 'large-capacity-final-create.py', 'large-capacity-final-create.log', 'large-capacity-final-verify.py', 'large-capacity-final-verify.log', 'large-capacity-final-snapshot-verification.json', 'large-capacity-snapshot-cod105-proof.log', 'large-capacity-snapshot-pairs5000-proof.log', 'large-capacity-snapshot-depth-cycles-proof.log', 'large-capacity-postmerge-comment.py', 'large-capacity-postmerge-comment.md', 'large-capacity-postmerge-comment.log', 'large-capacity-postmerge-ai-usage.json', 'large-capacity-postmerge-baseline-progress.json', 'large-capacity-baseline-disk-recovery-launch.json', 'large-capacity-baseline-recovery-completion.json', 'large-capacity-original-coordinator-enospc.log', 'large-capacity-recovered-coordinator-complete.log', 'large-capacity-adopt.py', 'large-capacity-adopt.log', 'large-capacity-adoption-preflight.json', 'large-capacity-final-pr-body.md', 'large-capacity-final-body-ai-usage.json', 'large-capacity-adoption-comment.md', 'large-capacity-adoption-comment.log', 'large-capacity-adoption-ai-usage.json', 'large-capacity-pr-patch.json', 'large-capacity-pr-patch-response.json', 'large-capacity-pr-ready.log', 'large-capacity-pr-merge.log', 'large-capacity-pr-merge.json', 'cover-choice-heap-space-sparse.py', 'cover-choice-heap-space-sparse.json', 'cover-choice-heap-space-sparse.log', 'cover-lns-space-sparse.py', 'cover-lns-space-sparse.json', 'cover-lns-space-sparse.log']];roots += [root/'artifacts/runs/large-capacity-final-20261118',root/'artifacts/runs/spectral-bound-final-20261118/config.json',root/'artifacts/runs/spectral-bound-final-20261118/OBSOLETE.json',root/'artifacts/runs/baseline-v2-20261115/config.json',root/'artifacts/runs/baseline-v2-20261115/exit.json',root/'experiments/large-capacity-snapshot-20261118.json'];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
