import os,sys,json,time,subprocess,resource,signal
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();cpu=int(sys.argv[1]);name=sys.argv[2];os.sched_setaffinity(0,{cpu});out=root/'artifacts/restoration-progress-selected'/name;out.mkdir(parents=True,exist_ok=False);records={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};r=records[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];binaries={'before':root/'artifacts/runs/dual-restoration-final-20261116/snapshot/bin/mip-codex','after':root/'target-dual-refinement/release/mip-codex'}
atomic_json(out/'experiment.json',dict(name=name,split='development',cpu=cpu,seconds=10,nodes=1,threads=1,memory_bytes=16*1024**3,input_sha256=r['sha256'],binaries={k:dict(path=str(p),sha256=sha256(p),build_info=json.loads(subprocess.check_output([str(p),'build-info'])))for k,p in binaries.items()},purpose='continuous root LP diagnostic; point dump is not an integer incumbent'))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for label,binary in binaries.items():
 d=out/label;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
 for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
 env['MIP_LP_TRACE']='1'
 env.update(MIP_ROOT_LP_DUMP=str(d/'root-lp.json'),OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');cmd=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'10','1'];start=time.monotonic()
 with (d/'stdout.log').open('w')as stdout,(d/'stderr.log').open('w')as stderr:
  p=subprocess.Popen(cmd,env=env,stdout=stdout,stderr=stderr,start_new_session=True,preexec_fn=restrict)
  try:p.wait(timeout=70);watchdog=False
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 execution=dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start,command=cmd);atomic_json(d/'execution.json',execution)
 if p.returncode==0:
  value=json.loads((d/'stdout.log').read_text());atomic_json(d/'result.json',value);print(name,label,{k:value.get(k)for k in ['status','objective','dual_bound','lp_pivots','lp_feasibility_pricing_calls','lp_pricing_fallbacks','seconds']},'lp_point',(d/'root-lp.json').exists(),flush=True)
 else:print(name,label,execution,flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
