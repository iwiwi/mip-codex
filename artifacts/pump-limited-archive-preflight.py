from pathlib import Path
import json,hashlib,tarfile,os
os.sched_setaffinity(0,{43});root=Path.cwd();records=[]
for name in ['pump-limited-pilot-evaluation','pump-limited-development-evaluation','pump-limited-default-correctness-evaluation','pump-limited-default-development-evaluation']:
 out=root/'artifacts'/name;manifest=json.load(open(out/'manifest.json'));lookup={r['path']:r for r in manifest};assert len(lookup)==len(manifest);seen=set()
 for p in sorted(out.glob('bundle-*.tar.gz')):
  with tarfile.open(p)as t:
   for m in t:
    assert m.name not in seen;seen.add(m.name);b=t.extractfile(m).read();r=lookup[m.name];assert len(b)==r['bytes']and hashlib.sha256(b).hexdigest()==r['sha256']
 assert seen==set(lookup)
 excluded=json.load(open(out/'excluded-elf.json'))
 for r in excluded:assert hashlib.sha256((root/r['path']).read_bytes()).hexdigest()==r['sha256']
 records.append(dict(archive=name,files=len(manifest),all_payload_sha256_valid=True,excluded_original_elf_sha256_valid=len(excluded)))
Path('artifacts/pump-limited-archive-preflight.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(records))
