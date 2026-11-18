from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/cover-long-round-correctness-controls-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/cover-long-round-frozen', 'artifacts/cover-long-round-origin.json', 'artifacts/cover-long-round-build.log', 'artifacts/cover-long-round-tests.log', 'artifacts/cover-long-round-test-summary.json', 'artifacts/cover-long-round-test-driver.py', 'artifacts/cover-long-round-test-driver.log', 'artifacts/cover-long-round-build-execution.json', 'artifacts/cover-long-round-enumeration', 'artifacts/cover-long-round-enumeration.log', 'artifacts/cover-long-round-large-generated', 'artifacts/cover-long-round-large-generated.py', 'artifacts/cover-long-round-large-generated.log', 'artifacts/cover-long-round-main-controls', 'artifacts/cover-long-round-main-controls.py', 'artifacts/cover-long-round-main-controls.log', 'artifacts/cover-long-round-long-controls', 'artifacts/cover-long-round-long-controls.py', 'artifacts/cover-long-round-long-controls.log', 'artifacts/cover-long-round-freeze.py', 'artifacts/cover-long-round-port.py', 'artifacts/cover-long-round-launch-controls.py', 'artifacts/cover-long-round-launch-controls.log', 'artifacts/cover-long-round-workers.json', 'docs/GRAPH_COVER_LONG_ROUNDS_HYPOTHESIS.md', 'experiments/cover-long-round-integer-bounds.json']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=353,rust_ignored=4,small_outputs=1200,small_gmp_valid=1200,small_proof_valid=1200,tamper_rejections=7410,large_outputs=12,large_gmp_valid=12,large_proof_valid=12,actual_long_mode_large_outputs=6,main_outputs=13,main_gmp_valid=13,main_proof_valid=13,hour_measurements_pending=True,new_record=False,candidate_source_changes=True,canonical_main_changed=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
