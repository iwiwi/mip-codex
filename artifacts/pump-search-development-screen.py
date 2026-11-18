"""Pinned own-solver development comparison; never reads a holdout model."""
import os,sys,json,time,hashlib,subprocess,resource,signal
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();cpu,shard,shards=map(int,sys.argv[1:4]);os.sched_setaffinity(0,{cpu});out=root/'artifacts/pump-search-development-screen';experiment=json.loads((out/'experiment.json').read_text());assert experiment['shards']==shards;manifest=[r for r in json.loads((root/'data/manifest.json').read_text())['instances']if r['split']=='development'];assert len(manifest)==188
for label,record in experiment['binaries'].items():assert sha256(Path(record['path']))==record['sha256']
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
records=[]
try:
 for k,r in enumerate(manifest):
  if k%shards!=shard:continue
  model=root/r['path'];assert sha256(model)==r['sha256'];labels=['before','after']if k%2==0 else['after','before']
  for label in labels:
   directory=out/'cases'/r['name']/label;directory.mkdir(parents=True,exist_ok=True);assert not(directory/'execution.json').exists(),'create-only screen';binary=Path(experiment['binaries'][label]['path']);env={k:v for k,v in os.environ.items()if not k.startswith("MIP_")};env.update(MIP_ADAPTIVE_RESOURCE_REPAIR='1',MIP_MIR_CUTS='1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');
   env.update(MIP_PUMP_TERMINAL_REPAIR='1',MIP_SEARCH='depth'if label=='before'else'best')
   cmd=['/usr/bin/time','-v','-o',str(directory/'resource.log'),str(binary),'solve',str(model),'10'];atomic_json(directory/'job.json',dict(command=cmd,input_sha256=r['sha256'],split='development',cpu=cpu,seconds=10,threads=1,memory_bytes=16*1024**3,wall_watchdog_seconds=70,environment={n:env[n]for n in ['MIP_SEARCH','MIP_PUMP_TERMINAL_REPAIR','MIP_ADAPTIVE_RESOURCE_REPAIR','MIP_MIR_CUTS','OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']if n in env}));start=time.monotonic();execution={};run=None
   with (directory/'stdout.log').open('w')as output,(directory/'stderr.log').open('w')as error:
    try:
     run=subprocess.Popen(cmd,env=env,stdout=output,stderr=error,preexec_fn=restrict,start_new_session=True);run.wait(timeout=70);execution.update(exit_code=run.returncode,watchdog=False)
    except subprocess.TimeoutExpired:
     os.killpg(run.pid,signal.SIGKILL);run.wait();execution.update(exit_code=run.returncode,watchdog=True)
    except BaseException:
     if run is not None and run.poll() is None:os.killpg(run.pid,signal.SIGKILL);run.wait()
     raise
   execution['wall_seconds']=time.monotonic()-start;summary=dict(name=r['name'],split='development',variant=label,cpu=cpu,status='driver_error',execution=execution,point_validation='pending independent audit')
   try:
    assert execution['exit_code']==0 and not execution['watchdog'];value=json.loads((directory/'stdout.log').read_text());atomic_json(directory/'result.json',value);summary.update({n:value[n]for n in ['status','objective','dual_bound','gap','nodes','seconds']});summary['pump']=value.get('pump');summary['cover_stats']=(value.get('binary_graph')or{}).get('cover_local');summary['graph_recognized']=value.get('binary_graph')is not None;summary['mir_added']=value['cut_stats'].get('mir_added',0);summary['mir_rejected']=value['cut_stats'].get('mir_rejected_cuts',0)
   except Exception as error:summary['error']=f'{type(error).__name__}: {error}'
   atomic_json(directory/'execution.json',execution);atomic_json(directory/'summary.json',summary);records.append(summary);atomic_json(out/f'worker-{shard}-progress.json',dict(completed=len(records),records=records));print(json.dumps(summary),flush=True)
 atomic_json(out/f'worker-{shard}-exit.json',dict(status='complete',completed=len(records),driver_errors=sum(r['status']=='driver_error'for r in records)))
except BaseException as error:
 atomic_json(out/f'worker-{shard}-exit.json',dict(status='error',completed=len(records),error=f'{type(error).__name__}: {error}'));raise
