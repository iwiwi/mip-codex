from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource
root=Path.cwd()
os.sched_setaffinity(0,{45})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
out=root/'artifacts/cover-heap-forget-comparison'
out.mkdir()
spec=json.load(open('artifacts/cover-heap-forget-source.json'))
assert sha256(root/'target/release/mip-codex')==spec['normal_cli_before_sha256']
variant=out/'forget-profile'
shutil.copy2(root/'target/release/examples/cover_heap_forget_profile',variant)
for name,h in spec['sources'].items():
    assert sha256(root/name)==h
    p=out/'source'/name
    p.parent.mkdir(parents=True,exist_ok=True)
    shutil.copy2(root/name,p)
original=root/'artifacts/cover-heap-fixed-work/heap-profile'
prior=json.load(open('artifacts/cover-heap-fixed-work/experiment.json'))
assert sha256(original)==prior['binaries']['heap']['sha256']
seed=root/'artifacts/cover-heap-long/single/result.json'
assert json.load(open(seed.parent/'validation.json'))['valid']
model=root/'data/raw/exploratory/z26.mps.gz'
binaries={'original':original,'mean64':variant}
atomic_json(out/'experiment.json',dict(cpu=45,threads=1,memory_bytes=16*1024**3,binaries={k:dict(path=str(p),sha256=sha256(p))for k,p in binaries.items()},variant_source=spec,input_sha256=sha256(model),seed_sha256=sha256(seed),seconds_each=300,iterations=100000000,target=None,stop_value=None,normal_cli_unchanged_after_example_build=True,purpose='same own seed/CPU/time original-vs-capped forgetting; private only'))
records=[]
for label,binary in binaries.items():
    d=out/label
    d.mkdir()
    env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
    env.update(MIP_DIAGNOSTIC_COVER_INITIAL='100000000',MIP_DIAGNOSTIC_COVER_EXTENDED='100000000')
    cmd=[str(binary),str(model),'300','single',str(d/'result.json'),str(seed)]
    atomic_json(d/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith('MIP_')}))
    start=time.monotonic()
    run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=480)
    (d/'stdout.log').write_text(run.stdout)
    (d/'stderr.log').write_text(run.stderr)
    atomic_json(d/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start))
    assert run.returncode==0,run.stderr
    metadata=json.loads(run.stdout)
    assert metadata['base_production_fingerprint']=='e5dfb6151bba081e'
    r=json.load(open(d/'result.json'))
    v=validate(model,r)
    atomic_json(d/'validation.json',v)
    assert v['valid']
    stats=r['rounds'][0]['stats']
    record=dict(label=label,objective=r['objective'],component_seconds=r['component_seconds'],iterations=stats['iterations'],forgettings=stats['forgettings'],gmp_valid=True)
    records.append(record)
    atomic_json(out/'records.json',records)
    print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=len(records),gmp_valid=len(records)))
