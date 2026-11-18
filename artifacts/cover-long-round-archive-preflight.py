from pathlib import Path
import os,json,hashlib,tarfile,subprocess,io
os.sched_setaffinity(0,{43});root=Path.cwd();names=['cover-long-round-correctness-controls-evaluation','cover-long-round-development-evaluation','cover-long-round-hour-before-native-evaluation','cover-long-round-hour-complete-evaluation','cover-long-round-default-correctness-evaluation','cover-long-round-default-development-evaluation','cover-long-round-default-long-evaluation','pump-terminal-postmerge-evaluation','pump-limited-postmerge-evaluation','large-primal-budget-postmerge-evaluation','cover-neutral-1198-evaluation','cover-parent-lns-1198-evaluation','cover-diverse-1198-reseed-evaluation','cover-vertex-penalty-correctness-evaluation','cover-vertex-penalty-long-evaluation'];records=[]
def read(p):
 if p.is_file():return p.read_bytes()
 return subprocess.check_output(['git','show','HEAD:'+str(p.relative_to(root))])
for name in names:
 d=root/'artifacts'/name;manifest=json.loads(read(d/'manifest.json'));lookup={r['path']:r for r in manifest};assert len(lookup)==len(manifest);seen=set();bundle_names={p.name for p in d.glob('bundle-*.tar.gz')}
 tracked=subprocess.check_output(['git','ls-tree','-r','--name-only','HEAD','--',str(d.relative_to(root))],text=True).splitlines();bundle_names.update(Path(p).name for p in tracked if Path(p).name.startswith('bundle-')and p.endswith('.tar.gz'))
 for f in sorted(bundle_names):
  with tarfile.open(fileobj=io.BytesIO(read(d/f)),mode='r:gz')as t:
   for m in t:
    assert m.name not in seen;seen.add(m.name);data=t.extractfile(m).read();r=lookup[m.name];assert len(data)==r['bytes']and hashlib.sha256(data).hexdigest()==r['sha256']
 assert seen==set(lookup),name
 p=d/'origin.json';origin=None
 if p.exists():origin=json.loads(p.read_text())
 elif any(x.endswith('/origin.json')for x in tracked):origin=json.loads(read(p))
 original=Path(origin['original_root'])if origin else Path('/home/ubuntu/mip-codex')if name=='large-primal-budget-postmerge-evaluation'else Path('/home/ubuntu/mip-cover-choice-heap')if name=='cover-neutral-1198-evaluation'else root
 excluded=json.loads(read(d/'excluded-elf.json'))
 for r in excluded:assert hashlib.sha256((original/r['path']).read_bytes()).hexdigest()==r['sha256'],(name,str(original/r['path']))
 records.append(dict(archive=name,files=len(manifest),bundles=len(bundle_names),all_payload_sha256_valid=True,excluded_elf_original_root=str(original),excluded_elf_sha256_valid=len(excluded)))
Path('artifacts/cover-long-round-archive-preflight.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(dict(archives=len(records),files=sum(r['files']for r in records),excluded_elf=sum(r['excluded_elf_sha256_valid']for r in records),all_valid=True)),flush=True)
