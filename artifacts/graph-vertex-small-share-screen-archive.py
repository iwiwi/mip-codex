from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/graph-vertex-small-share-screen-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/graph-vertex-small-share-development-screen', 'artifacts/graph-vertex-small-share-development-parallel-audit', 'artifacts/graph-vertex-small-share-development-analysis.json', 'artifacts/graph-vertex-small-share-connected', 'artifacts/graph-vertex-connected-native', 'artifacts/graph-vertex-small-share-short-controls', 'artifacts/graph-vertex-small-share-short-official', 'artifacts/graph-vertex-small-share-connected.py', 'artifacts/graph-vertex-connected-native.py', 'artifacts/graph-vertex-small-share-short-official.py', 'artifacts/graph-vertex-small-share-close.py', 'docs/GRAPH_VERTEX_SMALL_SHARE_RESULTS.md', 'scripts/comparison_worker.py']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,development_pairs=188,development_proof_valid=376,development_gmp_official_points=138,actual_vertex_development=0,connected_outputs=24,connected_gmp_proof_valid=24,connected_primal_improved=5,connected_primal_equal=6,connected_primal_worse=1,native_outputs=36,native_gmp_points=24,short_controls_gmp_proof_official=4,long_controls_pending=True,default_off=True,z26_300_regression_retained_in_separate_live_control_directory=True,new_record=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
