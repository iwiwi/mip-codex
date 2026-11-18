from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/general-recovery-correctness-evaluation';assert not out.exists()
paths=['artifacts/general-recovery-frozen', 'artifacts/general-recovery-origin.json', 'artifacts/general-recovery-version.json', 'artifacts/general-recovery-cli.sha256', 'artifacts/general-distance-tests.log', 'artifacts/general-recovery-build.log', 'artifacts/general-recovery-focused-tests.log', 'artifacts/general-recovery-focused-tests-initial.log', 'artifacts/general-recovery-focused-tests-v2.log', 'artifacts/general-recovery-focused-tests-v3.log', 'artifacts/general-recovery-full-tests.log', 'artifacts/general-recovery-pilot', 'artifacts/general-recovery-pilot.py', 'artifacts/general-recovery-pilot.log', 'artifacts/general-recovery-synthetic', 'artifacts/general-recovery-check-synthetic.py', 'artifacts/general-recovery-synthetic-worker.py', 'artifacts/general-recovery-synthetic-baseline.log', 'artifacts/general-recovery-synthetic-recovered.log', 'artifacts/general-recovery-synthetic-followup', 'artifacts/general-recovery-synthetic-followup.py', 'artifacts/general-recovery-synthetic-followup.log', 'artifacts/old-general-integer-pump-solve.rs', 'artifacts/old-general-integer-pump-revised.rs', 'docs/GENERAL_PUMP_LIMITED_RECOVERY_HYPOTHESIS.md', 'docs/GENERAL_PUMP_PREVIOUS_NEGATIVE.md'];roots=[root/p for p in paths];assert all(p.exists()for p in roots);files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=368,rust_ignored=4,focused_general_passed=4,distance_extension_passed=1,synthetic_each=200,synthetic_pass_each=198,same_main_numerical_errors_each=2,followup_gmp_valid=6,followup_official_unsupported=6,pilot_outputs=12,pilot_proof_valid=12,pilot_gmp_official_points=3,pilot_quality_improvements=0,default_off=True,adopted=False,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
