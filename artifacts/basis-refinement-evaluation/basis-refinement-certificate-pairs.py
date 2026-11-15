import json,os,subprocess,sys,resource,time
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import require_solver_origin,solver_source_fingerprint,sha256,atomic_json
from benchmark import verify_snapshot
os.sched_setaffinity(0,set(range(33,42)));os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1')
r=json.loads((root/'experiments/rational-certificate-snapshot-20261115.json').read_text());frozen=Path(r['directory']);config=json.loads((frozen/'config.json').read_text());verify_snapshot(frozen,config);old=frozen/'snapshot/bin/mip-codex';new=root/'target/release/mip-codex';old_info=json.loads(subprocess.check_output([str(old),'build-info']));new_info=require_solver_origin(root,new);assert old_info==config['solver_build_info']and old_info['source_fingerprint']==solver_source_fingerprint(frozen/'snapshot')and sha256(old)==r['binary_sha256']
out=root/'artifacts/basis-refinement-certificate-pairs';out.mkdir(exist_ok=True)
def restrict():
 os.sched_setaffinity(0,{0});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in ['glass4','app1-1']:
 model=root/f'data/raw/benchmark/{name}.mps.gz'
 for side,binary in [('before',old),('after',new)]:
  d=out/side/name;d.mkdir(parents=True,exist_ok=True);command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'10'];atomic_json(d/'experiment.json',dict(build_info=old_info if side=='before'else new_info,binary_sha256=sha256(binary),input_sha256=sha256(model),command=command,cpu=0,threads=1,memory_bytes=16*1024**3,split='development'))
  with(d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
  atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0;a=json.loads((d/'result.json').read_text());print(json.dumps(dict(side=side,name=name,**{k:a.get(k)for k in ['status','objective','dual_bound','nodes','seconds','lp_refinement_attempts','lp_refinement_steps','lp_refinement_successes','infeasibility_proof_failures']})),flush=True)
  if a.get('values')is not None:
   with(d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
   assert q.returncode==0 and all(json.loads((d/f'{label}-validation.json').read_text())['valid']is True for label in ['strict','official'])
  with(d/'proof.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(model),str(d/'result.json'),str(d/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
  assert q.returncode==0
