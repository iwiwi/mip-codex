from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/large-primal-budget-correctness-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/large-primal-budget-origin.json', 'artifacts/large-primal-budget-frozen', 'artifacts/large-primal-budget-build.log', 'artifacts/large-primal-budget-test-driver.py', 'artifacts/large-primal-budget-test-driver.log', 'artifacts/large-primal-budget-tests.log', 'artifacts/large-primal-budget-build-execution.json', 'artifacts/large-primal-budget-enumeration', 'artifacts/large-primal-budget-enumeration.log', 'artifacts/large-primal-budget-enumeration-checker-unavailable', 'artifacts/large-primal-budget-enumeration-checker-unavailable.log', 'artifacts/large-primal-budget-large-generated', 'artifacts/large-primal-budget-large-generated.log', 'artifacts/large-primal-budget-large-generated.py', 'artifacts/large-primal-budget-large-generated-initial', 'artifacts/large-primal-budget-large-generated-initial.log', 'artifacts/large-primal-budget-large-generated-initial.py', 'artifacts/large-primal-budget-freeze.py', 'artifacts/large-primal-budget-port.py', 'docs/GRAPH_LARGE_PRIMAL_BUDGET_HYPOTHESIS.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=353,rust_ignored=4,small_outputs=1200,small_gmp_valid=1200,small_proof_valid=1200,tamper_rejections=7410,large_outputs=12,large_gmp_valid=12,large_proof_valid=12,initial_unverified_small_outputs=1200,initial_large_driver_assertion_failures=1,new_record=False,candidate_source_changes=True,canonical_main_changed=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
