import json,os,resource,subprocess,time,hashlib
from pathlib import Path
out=Path('artifacts/mir-matrix-neos-root');out.mkdir(exist_ok=False);os.sched_setaffinity(0,{4});env=os.environ.copy()
for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP']:env['MIP_'+f]='0'
env['MIP_ROOT_LP_DUMP']=str((out/'root-lp.json').resolve());model=Path('data/raw/benchmark/neos-860300.mps.gz').resolve();cmd=[str(Path('target/release/mip-codex').resolve()),'solve',str(model),'60','1'];(out/'experiment.json').write_text(json.dumps(dict(split='development',command=cmd,env={k:v for k,v in env.items()if k.startswith('MIP_')},input_sha256=hashlib.sha256(model.read_bytes()).hexdigest(),build_info=json.loads(subprocess.check_output([cmd[0],'build-info'])),memory_bytes=16*1024**3,cpu=4),indent=2)+'\n')
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
start=time.monotonic()
with (out/'result.json').open('w')as output,(out/'stderr.log').open('w')as err:r=subprocess.run(cmd,stdout=output,stderr=err,env=env,preexec_fn=restrict,timeout=70)
(out/'execution.json').write_text(json.dumps(dict(exit_code=r.returncode,wall_seconds=time.monotonic()-start))+'\n');assert r.returncode==0;print(json.dumps(json.loads((out/'result.json').read_text())['dual_bound']))
