from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/ranged-rhs-correctness-evaluation';assert not out.exists()
roots=[p for p in(root/'artifacts').glob('ranged-rhs-*')if not any(p.name.startswith(x)for x in ['ranged-rhs-development','ranged-rhs-parallel-audit','ranged-rhs-correctness'])]
roots += [root/p for p in ['artifacts/implement-ranged-rhs-interval.py','artifacts/implement-ranged-rhs-interval-stage2.py','artifacts/add-ranged-rhs-tests.py','artifacts/add-prefix-provenance.py','artifacts/add-rhs-pair-telemetry.py','artifacts/extend-paired-rhs-verifier.py','docs/PAIRED_RHS_INTERVAL_HYPOTHESIS.md','scripts/verify_rhs_interval.py']]
assert all(p.exists()for p in roots);files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=368,rust_ignored=4,focused_pairs_passed=5,verifier_cases=14,tamper_rejected=11,exact_oracle_outputs=400,exact_oracle_errors=0,exact_rhs_verified=3484,exact_gmp_valid=106,pilot_two_versions_outputs=16,pilot_proof_valid=16,pilot_gmp_official_valid=8,diagnostic_prefix_rhs_verified=8,generic_binary_oracle_pass=1000,mixed_pass=195,mixed_preexisting_numerical_errors=5,mixed_extra_gmp_proof_valid=5,mixed_official_parser_unsupported=5,native_raw_outputs=600,native_glpk_raw_objsense_reader_failures=200,native_scip_original_gmp_invalid=157,native_highs_optimal_precision_discrepancies=1,glpk_normalized_equivalence_valid=200,glpk_normalized_points_gmp_valid=99,glpk_normalized_optimal_precision_discrepancies=1,development188_pending=True,default_off=True,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
