from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();out=root/'artifacts/cover-vertex-penalty-long-evaluation';assert not out.exists();roots=[root/p for p in ['artifacts/cover-vertex-penalty-v1', 'artifacts/cover-vertex-penalty-unseeded-v1', 'artifacts/cover-vertex-penalty-worker.py', 'artifacts/cover-vertex-penalty-launch.py', 'artifacts/cover-vertex-penalty-launch.log', 'artifacts/cover-vertex-penalty-unseeded-worker.py', 'artifacts/cover-vertex-penalty-results.json', 'docs/COVER_VERTEX_PENALTY_PLAN.md']];files=sorted({f for p in roots for f in([p]if p.is_file()else p.rglob('*'))if f.is_file()});manifest=[];excluded=[];groups=[];group=[];size=0
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
(out/'manifest.json').write_text(json.dumps(manifest,indent=2));(out/'excluded-elf.json').write_text(json.dumps(excluded,indent=2));s=dict(files=len(manifest),raw_bytes=sum(r['bytes']for r in manifest),compressed_bytes=sum(p.stat().st_size for p in out.glob('bundle-*.tar.gz')),bundles=len(groups),excluded_elf=len(excluded),all_payload_sha256_verified=True,outputs=8,gmp_valid=8,official_valid=8,seed_fresh_gmp_valid=4,seeded_objectives=[-1198]*4,unseeded_objectives=[-1186,-1126,-1130,-1129],new_record=False,production_changes=False);(out/'summary.json').write_text(json.dumps(s,indent=2));print(json.dumps(s))
