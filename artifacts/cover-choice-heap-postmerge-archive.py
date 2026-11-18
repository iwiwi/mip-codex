from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/cover-choice-heap-postmerge-evaluation';assert not out.exists();assert json.load(open('artifacts/cover-choice-heap-main-primary-audit/exit.json'))['gmp_valid']==4;roots=[root/p for p in ['artifacts/cover-choice-heap-main-update.py', 'artifacts/cover-choice-heap-main-update.json', 'artifacts/cover-choice-heap-main-update.log', 'artifacts/cover-choice-heap-main-build.log', 'artifacts/cover-choice-heap-main-smoke.py', 'artifacts/cover-choice-heap-main-smoke.log', 'artifacts/cover-choice-heap-main-smoke-system-python-failure.log', 'artifacts/cover-choice-heap-main-primary-audit', 'artifacts/cover-choice-heap-final-create.py', 'artifacts/cover-choice-heap-final-create.log', 'artifacts/cover-choice-heap-final-verify.py', 'artifacts/cover-choice-heap-final-verify.log', 'artifacts/cover-choice-heap-final-verify-initial.py', 'artifacts/cover-choice-heap-final-verify-initial.log', 'artifacts/cover-choice-heap-final-snapshot-verification.json', 'artifacts/cover-choice-heap-snapshot-cod105-proof.log', 'artifacts/cover-choice-heap-snapshot-pairs5000-proof.log', 'artifacts/cover-choice-heap-snapshot-depth-cycles-proof.log', 'artifacts/cover-choice-heap-postmerge-comment.py', 'artifacts/cover-choice-heap-postmerge-comment.md', 'artifacts/cover-choice-heap-postmerge-comment.log', 'artifacts/cover-choice-heap-postmerge-ai-usage.json', 'artifacts/large-capacity-native721-terminal.json', 'artifacts/runs/cover-choice-heap-final-20261118', 'artifacts/runs/large-capacity-final-20261118/config.json', 'artifacts/runs/large-capacity-final-20261118/OBSOLETE.json', 'experiments/cover-choice-heap-snapshot-20261118.json', 'artifacts/cover-choice-heap-main-adoption']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
