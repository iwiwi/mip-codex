import json,os,subprocess,sys,resource,time
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import sha256,require_solver_origin,atomic_json
os.sched_setaffinity(0,set(range(33,42)));binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary);model=root/'data/raw/benchmark/gen-ip002.mps.gz';out=root/'artifacts/joint-proof-final-fixed';out.mkdir(exist_ok=True)
atomic_json(out/'experiment.json',dict(build_info=info,binary_sha256=sha256(binary),input_sha256=sha256(model),seconds=30,nodes=100000,cpu=7,threads=1,memory_bytes=16*1024**3))
def restrict():
 os.sched_setaffinity(0,{7});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for enabled in [0,1]:
 d=out/str(enabled);d.mkdir(exist_ok=True);command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'30','100000'];atomic_json(d/'command.json',command)
 with (d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_JOINT_INFEASIBILITY=str(enabled)),preexec_fn=restrict,stdout=stream,stderr=err,timeout=45)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));r=json.loads((d/'result.json').read_text());assert p.returncode==0 and r['nodes']==100000 and r['joint_infeasibility_enabled']==bool(enabled)
 print(json.dumps(dict(enabled=enabled,**{k:r[k]for k in ['objective','dual_bound','status','nodes','seconds','lp_solves','lp_pivots','infeasibility_proof_trials','infeasibility_proof_failures']})),flush=True)
 for script,extra in [('benchmark_validate.py',[str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]),('verify_cover_cuts.py',[str(d/'proof.json')])]:
  with (d/(script+'.log')).open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts'/script),str(model),str(d/'result.json'),*extra],stdout=stream,stderr=subprocess.STDOUT)
  assert q.returncode==0
 assert all(json.loads((d/(label+'-validation.json')).read_text())['valid']is True for label in ['strict','official'])
a=json.loads((out/'0/result.json').read_text());b=json.loads((out/'1/result.json').read_text());assert all(a[k]==b[k]for k in ['objective','dual_bound','status','values','nodes','lp_solves','lp_pivots']);assert b['infeasibility_proof_failures']<a['infeasibility_proof_failures']
