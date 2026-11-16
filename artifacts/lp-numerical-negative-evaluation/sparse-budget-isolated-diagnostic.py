import json,os,sys,time,signal,subprocess,resource
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();cpu=41;os.sched_setaffinity(0,{cpu});out=root/'artifacts/sparse-budget-isolated-diagnostic';out.mkdir(exist_ok=False);manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};binary=root/'artifacts/runs/restoration-progress-final-20261116/snapshot/bin/mip-codex';records=[]
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for k,name in enumerate(['buildingenergy.mps.gz','ns1760995.mps.gz','square41.mps.gz','supportcase12.mps.gz','map10.mps.gz','neos-2075418-temuka.mps.gz']):
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256']
 for budget in ([512,2048]if k%2==0 else[2048,512]):
  d=out/name/str(budget);d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
  for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
  env.update(MIP_LP_WORKSPACE_MIB=str(budget),MIP_ROOT_LP_DUMP=str(d/'root-lp.json'),OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'10','1'];atomic_json(d/'experiment.json',dict(name=name,split='development',cpu=cpu,seconds=10,nodes=1,memory_bytes=16*1024**3,internal_budget_mib=budget,input_sha256=r['sha256'],binary_sha256=sha256(binary),command=cmd,environment={k:v for k,v in env.items()if k.startswith('MIP_')}));start=time.monotonic()
  with (d/'stdout.json').open('w')as output,(d/'stderr.log').open('w')as error:
   p=subprocess.Popen(cmd,env=env,stdout=output,stderr=error,start_new_session=True,preexec_fn=restrict)
   try:p.wait(timeout=70);watchdog=False
   except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
  atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start))
  record=dict(name=name,budget_mib=budget,exit_code=p.returncode,lp_point=(d/'root-lp.json').exists())
  if p.returncode==0:
   result=json.load(open(d/'stdout.json'));atomic_json(d/'result.json',result);record.update({k:result.get(k)for k in ['status','objective','dual_bound','lp_pivots','seconds']})
  records.append(record);print(json.dumps(record),flush=True);atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',records=len(records)))
