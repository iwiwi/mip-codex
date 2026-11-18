from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
cpu,pd=map(int,sys.argv[1:3]);root=Path.cwd();os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
from validate_solution import validate
out=root/'artifacts/cover-vertex-penalty-v1'/str(pd);out.mkdir();model=root/'data/raw/exploratory/z26.mps.gz';seed=Path(json.load(open('artifacts/cover-neutral-1198-v1/z26/audit-jobs.json'))[0]['point']);binary=root/'artifacts/cover-vertex-penalty-smoke/bin/profile';source=json.load(open('artifacts/cover-vertex-penalty-smoke/source.json'));assert sha256(binary)==source['profile_sha256'];assert sha256(seed)==source['seed_sha256'];v=validate(model,json.load(open(seed)));atomic_json(out/'seed-validation.json',v);assert v['valid'];cmd=[str(binary),str(model),'1800',str(pd),'149372736029117923','100000000',str(out/'result.json'),str(seed)];atomic_json(out/'job.json',dict(command=cmd,cpu=cpu,threads=1,memory_bytes=16*1024**3,input_sha256=sha256(model),seed_sha256=sha256(seed),binary_sha256=sha256(binary),target=None,stop_value=None,private_only=True));began=time.monotonic();watchdog=False
with(out/'stdout.log').open('w')as so,(out/'stderr.log').open('w')as se:
 p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True);atomic_json(out/'process.json',dict(pid=p.pid,process_group=p.pid,started_unix=time.time()))
 try:p.wait(timeout=1920)
 except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
atomic_json(out/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-began));assert p.returncode==0 and not watchdog
r=json.load(open(out/'result.json'));a=out/'audit';a.mkdir()
with(a/'audit.log').open('w')as log:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(out/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(a)],stdout=log,stderr=subprocess.STDOUT,timeout=600,check=True)
for kind in ['strict','official']:assert json.load(open(a/(kind+'-validation.json')))['valid']
s=dict(pd=pd,objective=r['objective'],seconds=r['component_seconds'],stats=r['stats'],gmp_valid=True,official_valid=True,seed_fresh_gmp_valid=True,private_only=True,global_bound_or_optimality_claim=False);atomic_json(out/'summary.json',s);print(json.dumps(s),flush=True)
