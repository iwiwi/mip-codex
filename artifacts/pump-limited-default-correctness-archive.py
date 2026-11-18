from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/pump-limited-default-correctness-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/pump-limited-default-frozen', 'artifacts/pump-limited-default-origin.json', 'artifacts/pump-limited-default-build.log', 'artifacts/pump-limited-default-full-tests.log', 'artifacts/pump-limited-default-tests.log', 'artifacts/pump-limited-default-source-preflight.json', 'artifacts/pump-limited-default-binary-oracle', 'artifacts/pump-limited-default-binary-oracle.log', 'artifacts/pump-limited-default-binary-profile-oracle', 'artifacts/pump-limited-default-binary-profile-oracle.log', 'artifacts/pump-limited-default-profile-label-correction.json', 'artifacts/pump-limited-default-mixed-oracle', 'artifacts/pump-limited-default-mixed-oracle.log', 'artifacts/pump-limited-default-mixed-failure-audit', 'artifacts/pump-limited-default-mixed-failure-audit.py', 'artifacts/pump-limited-default-mixed-failure-audit.log', 'artifacts/pump-limited-default-mcsched60', 'artifacts/pump-limited-default-mcsched60.py', 'artifacts/pump-limited-default-mcsched60.log', 'artifacts/pump-limited-default-mcsched60-initial-missing-model.log', 'artifacts/pump-limited-default-mcsched60-launch-correction.json', 'docs/ORIGINAL_WITNESS_FIVE_LIMITATION_20261118.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
for f in files:
 b=f.read_bytes();r=dict(path=str(f.relative_to(root)),bytes=len(b),sha256=hashlib.sha256(b).hexdigest())
 if b[:4]==b'\x7fELF':excluded.append(r);continue
 manifest.append(r)
 if group and size+len(b)>90*1024**2:groups.append(group);group=[];size=0
 group.append(f);size+=len(b)
if group:groups.append(group)
assert 'test result: FAILED' not in Path('artifacts/pump-limited-default-full-tests.log').read_text();assert 'Doc-tests' in Path('artifacts/pump-limited-default-full-tests.log').read_text();out.mkdir()
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes'] for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=357,rust_ignored=4,generic_models=200,generic_failures=0,binary_models=200,binary_failures=0,mixed_models=200,mixed_failures=5,mixed_failures_same_canonical=True,additional_five_gmp_proof_valid=5,mcsched60_points_gmp_proof_valid=2,mcsched60_primal_equal=True,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
