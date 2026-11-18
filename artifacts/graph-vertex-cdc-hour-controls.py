from pathlib import Path
import os,sys,json,subprocess,time,resource,signal,datetime
root=Path.cwd();os.sched_setaffinity(0,{46});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts')
from benchmark_common import sha256,atomic_json
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/graph-vertex-cdc-hour-controls';out.mkdir();model=root/'data/raw/exploratory/cdc7-4-3-2.mps.gz';binary=root/'artifacts/graph-vertex-frozen/bin/mip-codex';origin=json.load(open('artifacts/graph-vertex-hour-controls/experiment.json'))['binary'];assert sha256(binary)==origin['binary_sha256'];atomic_json(out/'experiment.json',dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),cpu=46,seconds=3600,watchdog=3660,threads=1,memory_bytes=16*1024**3,input_sha256=sha256(model),binary=origin,outside_all_240=True,seed=None,target=None,stop_value=None,formal_final_benchmark=False));records=[]
for flag in [0,1]:
 d=out/str(flag);d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_GRAPH_VERTEX_PORTFOLIO']=str(flag);cmd=[str(binary),'solve',str(model),'3600'];atomic_json(d/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith('MIP_')},cpu=46));start=time.monotonic();watchdog=False
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,env=env,start_new_session=True);atomic_json(d/'process.json',dict(pid=p.pid,process_group=p.pid,started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat()))
  try:p.wait(timeout=3660)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog));assert p.returncode==0 and not watchdog
 r=json.load(open(d/'stdout.log'));atomic_json(d/'result.json',r);v=validate(model,r);atomic_json(d/'validation.json',v);assert v['valid'];proof=validate_cut_proofs(model,r);atomic_json(d/'proof.json',proof);assert proof['valid'];records.append(dict(flag=flag,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],seconds=r['seconds'],cover_stats=r['binary_graph']['cover_local'],vertex_stats=r['binary_graph'].get('vertex_local'),gmp_valid=True,proof_valid=True));atomic_json(out/'records.json',records);print(json.dumps(records[-1]),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=2,gmp_valid=2,proof_valid=2))
