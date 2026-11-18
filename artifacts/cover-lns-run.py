from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource
root=Path.cwd();os.sched_setaffinity(0,{40})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
out=root/'artifacts/cover-lns-v1';out.mkdir()
spec=json.load(open('artifacts/cover-lns-source.json'))
assert sha256(root/'target/release/mip-codex')==spec['normal_cli_before_sha256']
binary=out/'profile';shutil.copy2(root/'target/release/examples/cover_lns_profile',binary)
for name,h in spec['sources'].items():
    assert sha256(root/name)==h
    p=out/'source'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/name,p)
model=root/'data/raw/exploratory/z26.mps.gz'
seed=root/'artifacts/cover-heap-long/single/result.json'
check=validate(model,json.load(open(seed)));atomic_json(out/'seed-validation.json',check);assert check['valid']
cmd=[str(binary),str(model),'600',str(out/'run'),str(seed)]
atomic_json(out/'experiment.json',dict(cpu=40,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),source=spec,input_sha256=sha256(model),seed_sha256=sha256(seed),command=cmd,target=None,stop_value=None,global_seconds=600,max_neighborhoods=200,seconds_per_neighborhood=2,nodes_per_neighborhood=200000,free_limit=1024,normal_cli_unchanged_after_example_build=True,purpose='own Rust exact restricted neighborhoods; primal only; no original global bound/optimality transfer'))
start=time.monotonic();r=subprocess.run(cmd,capture_output=True,text=True,timeout=900)
(out/'stdout.log').write_text(r.stdout);(out/'stderr.log').write_text(r.stderr)
atomic_json(out/'execution.json',dict(exit_code=r.returncode,wall_seconds=time.monotonic()-start));assert r.returncode==0,r.stderr
p=json.load(open(out/'run/profile.json'));assert p['base_production_fingerprint']=='e5dfb6151bba081e'and p['base_production_source_directory']==str(root)
records=[]
for point in sorted((out/'run').glob('*-original.json'))+list((out/'run').glob('result.json')):
    value=json.load(open(point));v=validate(model,value);atomic_json(point.with_name(point.stem+'-validation.json'),v);assert v['valid'];records.append(dict(kind='original',point=str(point.relative_to(out)),objective=value['objective'],gmp_valid=True))
for mps in sorted((out/'run').glob('*-restricted.mps')):
    point=mps.with_suffix('.json');value=json.load(open(point));v=validate(mps,value);atomic_json(point.with_name(point.stem+'-validation.json'),v);assert v['valid'];records.append(dict(kind='restricted',point=str(point.relative_to(out)),objective=value['objective'],gmp_valid=True))
atomic_json(out/'validations.json',records)
summary=dict(initial=p['initial'],best=p['best'],improvements=p['improvements'],seconds=p['seconds'],neighborhoods=len(p['records']),original_points_gmp=sum(r['kind']=='original'for r in records),restricted_points_gmp=sum(r['kind']=='restricted'for r in records),seed_revalidated=True,global_bound_or_optimality_claim=False)
atomic_json(out/'summary.json',summary);print(json.dumps(summary),flush=True)
