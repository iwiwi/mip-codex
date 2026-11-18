from pathlib import Path
import os,sys,json,subprocess,time,shutil,resource,signal
root=Path.cwd();os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/cover-vertex-penalty-smoke';out.mkdir();(out/'bin').mkdir();binary=out/'bin/profile';shutil.copy2(root/'target/release/examples/cover_vertex_penalty_profile',binary);seed=Path(json.load(open('artifacts/cover-neutral-1198-v1/z26/audit-jobs.json'))[0]['point']);atomic_json(out/'source.json',dict(profile_sha256=sha256(binary),rust_files={str(p):sha256(p)for p in [Path('examples/cover_vertex_penalty_profile.rs'),Path('examples/support/cover_vertex_penalty/mod.rs')]},base_rust_source_manifest={str(p):sha256(p)for p in sorted(Path('src').rglob('*.rs'))},normal_cli_sha256=sha256(root/'target/release/mip-codex'),cpu=40,threads=1,memory_bytes=16*1024**3,seed=str(seed),seed_sha256=sha256(seed)));records=[]
for label,model,seconds,point in [('cod105',root/'data/raw/benchmark/cod105.mps.gz',1,None),('pairs5000',Path('/home/ubuntu/mip-large-packing-profile/artifacts/large-cover-profile-v1/pairs5000.mps'),1,None),('z26-seeded',root/'data/raw/exploratory/z26.mps.gz',5,seed)]:
 d=out/label;d.mkdir();cmd=[str(binary),str(model),str(seconds),'5','149372736029117923','100000000',str(d/'result.json')]+([]if point is None else[str(point)]);atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model)));began=time.monotonic()
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True);watchdog=False
  try:p.wait(timeout=seconds+30)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-began));assert p.returncode==0 and not watchdog
 r=json.load(open(d/'result.json'));a=d/'audit';a.mkdir()
 with(a/'audit.log').open('w')as log:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(a)],stdout=log,stderr=subprocess.STDOUT,timeout=600,check=True)
 for kind in ['strict','official']:assert json.load(open(a/(kind+'-validation.json')))['valid']
 record=dict(label=label,objective=r['objective'],seconds=r['component_seconds'],stats=r['stats'],gmp_valid=True,official_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=3,gmp_valid=3,official_valid=3,private_only=True,new_record=False,production_changes=False))
