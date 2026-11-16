import json,os,resource,subprocess,sys,time,signal,statistics
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
cpu=int(sys.argv[1]);name=sys.argv[2];repeats=int(sys.argv[3]);os.sched_setaffinity(0,{cpu});root=Path.cwd();out=root/'artifacts'/('vec-backed-rhs-root-'+name);out.mkdir(exist_ok=False);name+='.mps.gz';r=next(r for r in json.load(open('data/manifest.json'))['instances']if r['name']==name);assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];binaries={k:root/'artifacts'/p/'bin/mip-codex'for k,p in [('before','parity-equality-final-diagnostic'),('after','vec-backed-rhs-v1-diagnostic')]};metadata={k:dict(path=str(p),sha256=sha256(p),build_info=json.loads(subprocess.check_output([str(p),'build-info'])))for k,p in binaries.items()};atomic_json(out/'experiment.json',dict(cpu=cpu,repeats=repeats,seconds_limit=60,threads=1,memory_bytes=16*1024**3,input_sha256=r['sha256'],binaries=metadata,purpose='pure own LP root, same algorithm, alternating old/new vector storage'))
env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
for f in ['PARITY_EQUALITIES','THREE_ROW_CUTS','MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
records=[]
for i in range(repeats):
 pair={}
 for label in (['before','after']if i%2==0 else['after','before']):
  d=out/f'{i:02}'/label;d.mkdir(parents=True);binary=binaries[label];assert sha256(binary)==metadata[label]['sha256'];e=dict(env,MIP_ROOT_LP_DUMP=str(d/'root-lp.json'));cmd=[str(binary),'solve',str(model),'60','1'];start=time.monotonic()
  with(d/'result.json').open('w')as f,(d/'stderr.log').open('w')as err:
   p=subprocess.Popen(cmd,stdout=f,stderr=err,env=e,preexec_fn=restrict,start_new_session=True)
   try:p.wait(timeout=120)
   except BaseException:
    if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
    raise
  assert p.returncode==0;v=json.load(open(d/'result.json'));pair[label]=v;record=dict(pair=i,label=label,wall_seconds=time.monotonic()-start,seconds=v['seconds'],status=v['status'],dual=v['dual_bound'],pivots=v['lp_pivots'],output_sha256=sha256(d/'result.json'));records.append(record);print(record,flush=True)
 def strip(v):
  if isinstance(v,dict):return{k:strip(x)for k,x in v.items()if 'seconds'not in k and k not in ['first_primal_seconds','lp_engine']}
  if isinstance(v,list):return[strip(x)for x in v]
  return v
 atomic_json(out/f'{i:02}/comparison.json',dict(common_json_identical=strip(pair['before'])==strip(pair['after'])))
 atomic_json(out/'records.json',records)
medians={k:statistics.median(r['seconds']for r in records if r['label']==k)for k in binaries};atomic_json(out/'summary.json',dict(repeats=repeats,medians=medians,ratio=medians['after']/medians['before'],common_json_identical=all(json.load(open(out/f'{i:02}/comparison.json'))['common_json_identical']for i in range(repeats))));print('SUMMARY',json.load(open(out/'summary.json')))
