from pathlib import Path
import os,sys,json,subprocess,time,resource
root=Path.cwd()
os.sched_setaffinity(0,{47})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
assert json.load(open('artifacts/cover-heap-long/summary.json'))['outputs']==2
seed=root/'artifacts/cover-heap-long/single/result.json'
assert json.load(open(seed.parent/'validation.json'))['valid']
binary=root/'artifacts/cover-heap-fixed-work/heap-profile'
prior=json.load(open('artifacts/cover-heap-fixed-work/experiment.json'))
assert sha256(binary)==prior['binaries']['heap']['sha256']
model=root/'data/raw/exploratory/z26.mps.gz'
out=root/'artifacts/cover-heap-continuation-900'
out.mkdir()
env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
env.update(MIP_DIAGNOSTIC_COVER_INITIAL='100000000',MIP_DIAGNOSTIC_COVER_EXTENDED='100000000')
cmd=[str(binary),str(model),'900','reseed',str(out/'result.json'),str(seed)]
atomic_json(out/'experiment.json',dict(cpu=47,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),input_sha256=sha256(model),seed_sha256=sha256(seed),command=cmd,environment={k:v for k,v in env.items()if k.startswith('MIP_')},target=None,stop_value=None,purpose='exploratory continuation from best own GMP-validated point; full long rounds before random reseed; no bounds/main-performance claim'))
start=time.monotonic()
run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=1080)
(out/'stdout.log').write_text(run.stdout)
(out/'stderr.log').write_text(run.stderr)
atomic_json(out/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start))
assert run.returncode==0,run.stderr
r=json.load(open(out/'result.json'))
v=validate(model,r)
atomic_json(out/'validation.json',v)
assert v['valid']
record=dict(objective=r['objective'],component_seconds=r['component_seconds'],rounds=len(r['rounds']),iterations=sum(q['stats']['iterations']for q in r['rounds']),gmp_valid=True)
atomic_json(out/'summary.json',record)
print(json.dumps(record),flush=True)
