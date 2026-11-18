from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{44});root=Path.cwd();out=root/'artifacts/general-last-projection-evaluation';assert not out.exists()
paths=['artifacts/general-last-projection-frozen', 'artifacts/general-last-projection-version.json', 'artifacts/general-last-projection-cli.sha256', 'artifacts/general-last-projection-origin.json', 'artifacts/general-last-projection-focused-tests.log', 'artifacts/general-last-projection-focused-tests-v2.log', 'artifacts/general-last-projection-build.log', 'artifacts/general-last-projection-full-tests.log', 'artifacts/general-last-projection-freeze.py', 'artifacts/general-last-projection-pilot', 'artifacts/general-last-projection-pilot.py', 'artifacts/general-last-projection-pilot.log', 'artifacts/general-last-projection-comparison.json', 'artifacts/general-recovery-structural-screen', 'artifacts/general-recovery-structural-screen.py', 'artifacts/general-recovery-structural-screen.log', 'artifacts/general-recovery-structural-screen-initial.log', 'artifacts/general-recovery-structural-screen-v2.log', 'artifacts/general-recovery-structural-screen-v3.log', 'artifacts/general-recovery-structural-pilot', 'artifacts/general-recovery-structural-pilot.py', 'artifacts/general-recovery-structural-pilot.log', 'docs/GENERAL_PUMP_LIMITED_RECOVERY_HYPOTHESIS.md', 'docs/GENERAL_PUMP_RECOVERY_NEGATIVE_RESULTS.md'];roots=[root/p for p in paths];assert all(p.exists()for p in roots);files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=369,rust_ignored=4,focused_passed=5,v1_structural_outputs=12,v1_structural_proof_valid=12,v1_structural_gmp_official=12,v2_outputs=24,v2_proof_valid=24,v2_gmp_official=15,v2_last_projection_repairs=4,v2_last_projection_candidates=0,all8_quality_unchanged=True,default_off=True,adopted=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
