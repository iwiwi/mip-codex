from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{53});root=Path.cwd();out=root/'artifacts/farkas-row-projection-default-correctness-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/farkas-row-projection-default-frozen', 'artifacts/farkas-row-projection-default-origin.json', 'artifacts/farkas-row-projection-default-parent.json', 'artifacts/farkas-row-projection-default-rust-diff.patch', 'artifacts/farkas-row-projection-default-build.log', 'artifacts/farkas-row-projection-default-tests.log', 'artifacts/farkas-row-projection-default-test-summary.json', 'artifacts/farkas-row-projection-default-full-tests.py', 'artifacts/farkas-row-projection-default-freeze.py', 'artifacts/farkas-row-projection-default-synthetic', 'artifacts/farkas-row-projection-default-synthetic-worker.py', 'artifacts/farkas-row-projection-default-check-synthetic.py', 'artifacts/farkas-row-projection-default-mixed-limitations-audit-v2', 'artifacts/farkas-row-projection-default-mixed-limitations-audit.py', 'artifacts/farkas-row-projection-default-mixed-audit-wait.py', 'docs/FARKAS_ROW_PROJECTION_HYPOTHESIS.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,default_on_without_environment_override=True,rust_passed=363,rust_ignored=4,generic_binary_oracle_pass=1000,mixed_oracle_pass=195,mixed_same_preexisting_numerical_errors=5,mixed_extra_gmp_proof_points_valid=5,mixed_official_parser_unsupported=5,initial_to_final_rust_diff_only_default_and_trial_counter=True,development188_pending=True,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
