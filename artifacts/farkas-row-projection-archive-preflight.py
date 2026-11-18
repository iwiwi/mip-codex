from pathlib import Path
import json,hashlib,tarfile,os,subprocess,io
os.sched_setaffinity(0,{53});root=Path.cwd();names=['numerical-cold-evaluation','numerical-cold-replay-evaluation','farkas-row-projection-correctness-evaluation','farkas-row-projection-development-evaluation','farkas-row-projection-long-evaluation','farkas-row-projection-default-correctness-evaluation','farkas-row-projection-default-development-evaluation']
def read(p):
 if p.exists():return p.read_bytes()
 return subprocess.check_output(['git','show','HEAD:'+str(p.relative_to(root))],stderr=subprocess.DEVNULL)
records=[]
for name in names:
 folder=root/'artifacts'/name;manifest=json.loads(read(folder/'manifest.json'));lookup={r['path']:r for r in manifest};seen=set();bundles=[]
 # All currently generated bundles are present; Git fallback supports sparse checkout later.
 available=sorted(folder.glob('bundle-*.tar.gz'))
 if not available:
  paths=subprocess.check_output(['git','ls-files',str(folder.relative_to(root))+'/bundle-*.tar.gz'],text=True).splitlines();available=[root/p for p in paths]
 assert available,name
 for path in available:
  data=read(path);bundles.append(dict(path=str(path.relative_to(root)),bytes=len(data),sha256=hashlib.sha256(data).hexdigest()))
  with tarfile.open(fileobj=io.BytesIO(data),mode='r:gz')as archive:
   for member in archive:
    assert member.isfile()and member.name not in seen;seen.add(member.name);b=archive.extractfile(member).read();r=lookup[member.name];assert len(b)==r['bytes']and hashlib.sha256(b).hexdigest()==r['sha256'],member.name
 assert seen==set(lookup),name
 try:origin=json.loads(read(folder/'origin.json'))
 except subprocess.CalledProcessError:origin={}
 original=Path(origin.get('original_root',str(root)));excluded=json.loads(read(folder/'excluded-elf.json'))
 for r in excluded:
  p=original/r['path'];assert p.is_file()and p.stat().st_size==r['bytes']and hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256'],str(p)
 records.append(dict(archive=name,payload_files=len(manifest),excluded_elf=len(excluded),original_root=str(original),bundles=bundles,all_sha256_verified=True))
flat=root/'artifacts/evaluation-report-postmerge-evaluation';manifest=json.loads(read(flat/'manifest.json'))
for name,sha in manifest.items():assert hashlib.sha256(read(flat/name)).hexdigest()==sha,name
summary=dict(archives=records,payload_files=sum(r['payload_files']for r in records),excluded_elf=sum(r['excluded_elf']for r in records),postmerge_flat_payloads=len(manifest),all_payload_and_excluded_elf_sha256_verified=True,excluded_elf_checked_at_original_roots=True);(root/'artifacts/farkas-row-projection-archive-preflight.json').write_text(json.dumps(summary,indent=2));print(json.dumps({k:v for k,v in summary.items()if k!='archives'}))
