from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
cpu,shard=map(int,sys.argv[1:3]);root=Path.cwd();os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json
from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/pump-terminal-pilot';jobs=json.load(open(out/'jobs.json'));binary=out/'frozen/bin/mip-codex';assert sha256(binary)==json.load(open(out/'origin.json'))['binary_sha256'];records=[]
for i,job in enumerate(jobs):
 if i%2!=shard:continue
 model=root/job['path'];assert sha256(model)==job['sha256']
 for flag in ([0,1]if i%2==0 else[1,0]):
  d=out/'cases'/job['name']/str(flag);d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_PUMP_TERMINAL_REPAIR=str(flag),MIP_ADAPTIVE_RESOURCE_REPAIR='1',MIP_MIR_CUTS='1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1');cmd=[str(binary),'solve',str(model),'10'];atomic_json(d/'job.json',dict(command=cmd,cpu=cpu,memory_bytes=16*1024**3,threads=1,input_sha256=job['sha256'],environment={k:v for k,v in env.items()if k.startswith('MIP_')}));start=time.monotonic();watchdog=False
  with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
   p=subprocess.Popen(cmd,stdout=so,stderr=se,env=env,start_new_session=True);atomic_json(d/'process.json',dict(pid=p.pid,process_group=p.pid,started_unix=time.time()))
   try:p.wait(timeout=70)
   except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
  atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog));assert p.returncode==0 and not watchdog
  r=json.load(open(d/'stdout.log'));atomic_json(d/'result.json',r);v=None
  if r.get('values')is not None:
   a=d/'audit';a.mkdir()
   with(a/'audit.log').open('w')as log:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(a)],stdout=log,stderr=subprocess.STDOUT,timeout=600,check=True)
   v=json.load(open(a/'strict-validation.json'));official=json.load(open(a/'official-validation.json'));assert v['valid'] and official['valid']
  proof=validate_cut_proofs(model,r);atomic_json(d/'proof.json',proof);assert proof['valid'];records.append(dict(name=job['name'],flag=flag,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],seconds=r['seconds'],pump=r['pump'],first_primal_seconds=r.get('first_primal_seconds'),gmp_valid=None if v is None else True,official_valid=None if v is None else True,proof_valid=True));atomic_json(out/f'worker-{shard}-records.json',records);print(json.dumps(records[-1]),flush=True)
atomic_json(out/f'worker-{shard}-exit.json',dict(status='complete',outputs=len(records),gmp_valid=sum(r['gmp_valid']is True for r in records),proof_valid=len(records)))
