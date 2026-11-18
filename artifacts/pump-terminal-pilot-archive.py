from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/pump-terminal-pilot-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/pump-terminal-pilot', 'artifacts/pump-terminal-pilot-launch.py', 'artifacts/pump-terminal-pilot-worker.py', 'artifacts/pump-terminal-pilot-launch.log', 'artifacts/pump-terminal-pilot-analysis.py', 'artifacts/pump-terminal-pilot-analysis.log', 'artifacts/pump-terminal-pilot-analysis-initial-not-ready.json', 'artifacts/pump-terminal-focused-tests.log', 'artifacts/pump-terminal-tests.log', 'artifacts/pump-terminal-test-summary.json', 'artifacts/pump-terminal-build.log', 'artifacts/pump-terminal-port.py', 'artifacts/pump-terminal-binary-oracle', 'artifacts/pump-terminal-binary-oracle.log', 'artifacts/pump-terminal-native-drayage-25-23', 'artifacts/pump-terminal-native-drayage-25-23.py', 'artifacts/pump-terminal-native-drayage-25-23.log', 'artifacts/pump-terminal-native-graph20-20-1rand', 'artifacts/pump-terminal-native-graph20-20-1rand.py', 'artifacts/pump-terminal-native-graph20-20-1rand.log', 'artifacts/pump-terminal-h80-repeats', 'artifacts/pump-terminal-h80-repeats.log', 'artifacts/pump-terminal-h80-worker.py', 'artifacts/pump-candidate-descent-v1', 'artifacts/pump-candidate-descent-run.py', 'artifacts/pump-candidate-descent-run.log', 'artifacts/pump-candidate-descent-build.log', 'artifacts/pump-descent-normal-cli-before.sha256', 'examples/pump_candidate_descent.rs', 'docs/PUMP_TERMINAL_REPAIR_HYPOTHESIS.md', 'docs/PUMP_CANDIDATE_DESCENT_HYPOTHESIS.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_passed=360,rust_ignored=4,binary_cases=200,binary_failures=0,pilot_outputs=34,pilot_proof_valid=34,pilot_gmp_official_valid=10,pilot_primal_same=14,pilot_primal_improved=2,pilot_primal_worsened=1,native_outputs=6,native_gmp_valid=6,h80_repetition_outputs=6,h80_repetition_proof_valid=6,h80_repetition_gmp_valid=3,descent_diagnostic_outputs=6,descent_gmp_official_valid=6,descent_improved=1,descent_seed_validations_reused=6,canonical_main_changed=False,new_record=False,full_development_pending=True);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
