import json,os,resource,subprocess,sys,time,signal
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
cpu=int(sys.argv[1]);name=sys.argv[2];row='0';os.sched_setaffinity(0,{cpu});root=Path.cwd();out=root/'artifacts'/f'phase-one-delayed-pricing-root-{name}';out.mkdir(exist_ok=False);r=next(r for r in json.load(open('data/manifest.json'))['instances']if r['name']==name+'.mps.gz');assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];binary=root/'artifacts/phase-one-delayed-pricing-diagnostic/bin/mip-codex';atomic_json(out/'experiment.json',dict(cpu=cpu,rows=row,seconds=60,threads=1,memory_bytes=16*1024**3,input_sha256=r['sha256'],binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),purpose='pure LP Phase-I entering-rule comparison; row dominance independently toggled'))
env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
for f in ['PARITY_EQUALITIES','THREE_ROW_CUTS','MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
records=[]
for flag,keep,delay in [('off','0','0'),('immediate','1','0'),('delayed','1','1')]:
 d=out/flag;d.mkdir();e=dict(env,MIP_EXACT_ROW_DOMINANCE=row,MIP_PHASE1_KEEP_PRICING=keep,MIP_PHASE1_DELAYED_PRICING=delay,MIP_LP_TRACE='1',MIP_ROOT_LP_DUMP=str(d/'root-lp.json'));atomic_json(d/'environment.json',{k:v for k,v in e.items()if k.startswith('MIP_')or k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']});cmd=[str(binary),'solve',str(model),'60','1'];start=time.monotonic()
 with(d/'result.json').open('w')as f,(d/'stderr.log').open('w')as err:
  p=subprocess.Popen(cmd,env=e,stdout=f,stderr=err,preexec_fn=restrict,start_new_session=True)
  try:p.wait(timeout=120)
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 assert p.returncode==0;v=json.load(open(d/'result.json'));r=dict(flag=flag,status=v['status'],seconds=v['seconds'],wall_seconds=time.monotonic()-start,pivots=v['lp_pivots'],dual=v['dual_bound']);records.append(r);atomic_json(out/'records.json',records);print(r,flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
