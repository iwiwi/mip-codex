import json,os,resource,subprocess,sys
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import require_solver_origin,sha256,atomic_json
os.sched_setaffinity(0,set(range(33,42)))
core=int(sys.argv[1]);names=sys.argv[2:];binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1')
for key in ['MIP_BINARY_GRAPH','MIP_COVER_CUTS','MIP_ROUNDING_CUTS','MIP_LP_DIVING','MIP_PRIMAL_REPAIR','MIP_PRIMAL_PROBE','MIP_RINS','MIP_FEASIBILITY_PUMP','MIP_STRONG_BRANCHING','MIP_OBJECTIVE_ROUNDING','MIP_PROPAGATION']:env[key]='0'
def restrict():
 os.sched_setaffinity(0,{core});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in names:
 record=manifest[name];assert record['split']=='development';model=root/record['path'];assert sha256(model)==record['sha256']
 d=root/'artifacts/aggregate-cut-roots'/name.removesuffix('.mps.gz');d.mkdir(parents=True,exist_ok=True)
 command=[str(binary),'solve',str(model),'10','1'];point=d/'root-lp.json'
 atomic_json(d/'experiment.json',dict(command=command,input_sha256=record['sha256'],split=record['split'],build_info=info,binary_sha256=sha256(binary),cpu=core,threads=1,memory_bytes=16*1024**3,environment={k:v for k,v in env.items()if k.startswith('MIP_')},purpose='diagnostic LP relaxation; no new optimizer cuts'))
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:
  try:p=subprocess.run(command,env=dict(env,MIP_ROOT_LP_DUMP=str(point)),preexec_fn=restrict,stdout=out,stderr=err,timeout=60);status=dict(exit_code=p.returncode)
  except subprocess.TimeoutExpired:status=dict(external_timeout=True,seconds=60)
 atomic_json(d/'exit.json',status)
 try:r=json.loads((d/'result.json').read_text());summary=dict(name=name,status=r['status'],point_dump=point.is_file(),objective=r['objective'],dual_bound=r['dual_bound'])
 except Exception as e:summary=dict(name=name,point_dump=point.is_file(),error=str(e))
 if point.is_file():
  data=json.loads(point.read_text());assert len(data['names'])==len(data['point'])and len(set(data['names']))==len(data['names']);summary['lp_objective']=data['objective']
 if summary.get('objective')is not None:
  with(d/'validation.log').open('w')as out:q=subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'third_party/miplib-checker/bin/solchecker',str(d)],stdout=out,stderr=subprocess.STDOUT)
  assert q.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True
 atomic_json(d/'summary.json',summary);print(json.dumps(summary),flush=True)
