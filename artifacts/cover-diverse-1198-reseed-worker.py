from pathlib import Path
import sys,os,json,subprocess,time,resource,signal
cpu=int(sys.argv[1]);seed=Path(sys.argv[2]);out=Path(sys.argv[3]);os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');root=Path.cwd();sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
out.mkdir();model=root/'data/raw/exploratory/z26.mps.gz';spec=json.load(open('artifacts/cover-heap-fixed-work/experiment.json'))['binaries']['heap'];binary=Path(spec['path']);assert sha256(binary)==spec['sha256'];v=validate(model,json.load(open(seed)));atomic_json(out/'seed-validation.json',v);assert v['valid']
env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_DIAGNOSTIC_COVER_INITIAL='100000000',MIP_DIAGNOSTIC_COVER_EXTENDED='100000000');cmd=[str(binary),str(model),'1800','reseed',str(out/'result.json'),str(seed)];atomic_json(out/'job.json',dict(command=cmd,cpu=cpu,threads=1,memory_bytes=16*1024**3,seed_sha256=sha256(seed),model_sha256=sha256(model),binary=spec,iteration_cap=100000000,seconds_guard=1800,target=None,stop_value=None,private_only=True))
start=time.monotonic()
with(out/'stdout.log').open('w')as so,(out/'stderr.log').open('w')as se:
 p=subprocess.Popen(cmd,stdout=so,stderr=se,env=env,start_new_session=True);atomic_json(out/'process.json',dict(pid=p.pid,process_group=p.pid,started_unix=time.time()))
 try:p.wait(timeout=1920)
 except subprocess.TimeoutExpired:
  os.killpg(p.pid,signal.SIGKILL);p.wait();atomic_json(out/'execution.json',dict(exit_code=p.returncode,watchdog=True));raise
atomic_json(out/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=False));assert p.returncode==0
r=json.load(open(out/'result.json'));v=validate(model,r);atomic_json(out/'validation.json',v);assert v['valid'];atomic_json(out/'summary.json',dict(objective=r['objective'],component_seconds=r['component_seconds'],gmp_valid=True,private_only=True,original_bound_or_optimality_claim=False));print(json.dumps(json.load(open(out/'summary.json'))),flush=True)
