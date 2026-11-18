from pathlib import Path
import os,subprocess,json,hashlib,shutil,datetime
root=Path.cwd();os.sched_setaffinity(0,{54});targets=[Path('/home/ubuntu/mip-zero-phase-one/target/debug/build'),Path('/home/ubuntu/mip-zero-phase-one/target/debug/incremental')]
assert not any(subprocess.run(['pgrep','-x',n],capture_output=True).returncode==0 for n in ['cargo','rustc'])
patterns=['artifacts/**/manifest.json','artifacts/**/excluded-elf.json','artifacts/**/*SHA256*.json','experiments/**'];cmd=['git','grep','--cached','-l','-F']
for p in targets:cmd+=['-e',str(p)]
r=subprocess.run(cmd+['--']+patterns,capture_output=True,text=True);assert r.returncode==1 and not r.stdout,r.stdout
for p in targets:assert p.is_dir()and not p.is_symlink()and p.name in ['build','incremental']and p.parent.name=='debug'
for f in Path('/proc').glob('[0-9]*/exe'):
 try:e=os.readlink(f)
 except OSError:continue
 assert not any(e.startswith(str(p)+'/')for p in targets),e
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();keep=[]
for target in sorted({p.parent.parent for p in targets}):
 for p in target.rglob('*'):
  if not p.is_file()or p.is_symlink()or any(p.is_relative_to(t)for t in targets):continue
  with p.open('rb')as f:magic=f.read(4)
  if magic==b'\x7fELF':keep.append(dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p)))
protected=[root/'target/release/mip-codex',root/'artifacts/runs/cover-long-round-final-20261118/config.json',root/'artifacts/runs/baseline-v2-20261115/config.json',root/'artifacts/runs/baseline-v2-20261115/results.jsonl',root/'artifacts/runs/baseline-v2-disk-recovery-20261118/config.json'];before={str(p):sha(p)for p in protected};free=shutil.disk_usage('/').free;removed=[]
for p in targets:
 assert not any(subprocess.run(['pgrep','-x',n],capture_output=True).returncode==0 for n in ['cargo','rustc'])
 size=int(subprocess.check_output(['du','-sb',str(p)],text=True).split()[0]);shutil.rmtree(p);removed.append(dict(path=str(p),logical_bytes=size))
for r in keep:assert Path(r['path']).stat().st_size==r['bytes']and sha(Path(r['path']))==r['sha256']
assert before=={str(p):sha(p)for p in protected}
r=dict(utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),removed_regenerable_build_caches=removed,logical_bytes=sum(x['logical_bytes']for x in removed),free_before=free,free_after=shutil.disk_usage('/').free,retained_noncache_elfs=keep,retained_elf_sha256_verified=True,protected_snapshot_baseline_cli_sha256=before,tracked_manifest_reference_matches=0,no_compiler_or_process_in_deleted_paths=True)
(root/'artifacts/evaluation-report-shared-debug-cleanup.json').write_text(json.dumps(r,indent=2));print(json.dumps({k:v for k,v in r.items()if k not in ['retained_noncache_elfs','protected_snapshot_baseline_cli_sha256']}))
