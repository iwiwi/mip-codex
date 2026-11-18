from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource
root=Path.cwd()
os.sched_setaffinity(0,{40})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
out=root/'artifacts/cover-parent-recombine'
out.mkdir()
spec=json.load(open('artifacts/cover-parent-recombine-source.json'))
assert sha256(root/'target/release/mip-codex')==spec['normal_cli_before_sha256']
binary=out/'profile'
shutil.copy2(root/'target/release/examples/cover_parent_recombine',binary)
for name,h in spec['sources'].items():
    assert sha256(root/name)==h
    p=out/'source'/name
    p.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(root/name,p)
model=root/'data/raw/exploratory/z26.mps.gz'
large=Path('/home/ubuntu/mip-large-packing-profile')
parents=[large/'artifacts/large-cover-profile-v1/z26-single/result.json',large/'artifacts/large-cover-profile-v1/z26-reseed/result.json',large/'artifacts/large-cover-continuation/single/result.json',large/'artifacts/large-cover-continuation/reseed/result.json',root/'artifacts/cover-heap-fixed-time/0/heap/result.json',root/'artifacts/cover-heap-fixed-time/1/heap/result.json',root/'artifacts/cover-heap-fixed-time/2/heap/result.json',root/'artifacts/cover-heap-long/single/result.json',root/'artifacts/cover-heap-long/reseed/result.json']
records=[]
for k,p in enumerate(parents):
    point=json.load(open(p))
    v=validate(model,point)
    atomic_json(out/f'parent-{k:02d}-validation.json',v)
    assert v['valid']
    records.append(dict(path=str(p),sha256=sha256(p),objective=point['objective'],gmp_valid=True))
atomic_json(out/'experiment.json',dict(cpu=40,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),source=spec,input_sha256=sha256(model),parents=records,seconds=60,target=None,stop_value=None,normal_cli_unchanged_after_example_build=True,purpose='own Rust parent points only; exact bipartite union matching gives a primal candidate only'))
d=out/'run'
cmd=[str(binary),str(model),'60',str(d),*map(str,parents)]
atomic_json(out/'job.json',dict(command=cmd))
start=time.monotonic()
run=subprocess.run(cmd,capture_output=True,text=True,timeout=180)
(out/'stdout.log').write_text(run.stdout)
(out/'stderr.log').write_text(run.stderr)
atomic_json(out/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start))
assert run.returncode==0,run.stderr
metadata=json.load(open(d/'profile.json'))
assert metadata['base_production_fingerprint']=='e5dfb6151bba081e'
assert metadata['base_production_source_directory']==str(root)
r=json.load(open(d/'result.json'))
v=validate(model,r)
atomic_json(d/'validation.json',v)
assert v['valid']
summary=dict(parents=len(parents),parents_gmp_valid=len(parents),unique_pool=metadata['pool_size'],pairs=len(metadata['records']),initial=metadata['initial'],best=metadata['best'],objective=r['objective'],gmp_valid=True,seconds=metadata['seconds'],global_bound_or_optimality_claim=False)
atomic_json(out/'summary.json',summary)
print(json.dumps(summary),flush=True)
