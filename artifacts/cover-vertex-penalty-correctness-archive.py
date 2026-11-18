from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/cover-vertex-penalty-correctness-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/cover-vertex-penalty-smoke', 'artifacts/cover-vertex-penalty-smoke.py', 'artifacts/cover-vertex-penalty-smoke.log', 'artifacts/cover-vertex-penalty-tests.log', 'artifacts/cover-vertex-penalty-build.log', 'artifacts/cover-vertex-penalty-initial-penalty-width', 'artifacts/cover-vertex-normal-cli-before.sha256', 'examples/cover_vertex_penalty_profile.rs', 'examples/support/cover_vertex_penalty/mod.rs', 'docs/COVER_VERTEX_PENALTY_PLAN.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,rust_tests=5,exhaustive_graphs=200,penalty_delays_per_graph=4,incremental_random_graphs=100,initial_failed_tests=2,initial_source_and_log_retained=True,smoke_outputs=3,gmp_valid=3,official_valid=3,long_four_settings_pending=True,production_changes=False,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
