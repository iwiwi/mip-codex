import json,os,subprocess,sys,time,resource
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import require_solver_origin,solver_source_fingerprint,sha256,atomic_json
from benchmark import verify_snapshot
os.sched_setaffinity(0,set(range(33,42)))
receipt=json.loads((root/'experiments/graph-adaptive-snapshot-20261115.json').read_text());frozen=Path(receipt['directory']);config=json.loads((frozen/'config.json').read_text());verify_snapshot(frozen,config)
old=frozen/'snapshot/bin/mip-codex';new=root/'target/release/mip-codex';old_info=json.loads(subprocess.check_output([str(old),'build-info']));new_info=require_solver_origin(root,new)
assert old_info==config['solver_build_info'] and old_info['source_fingerprint']==solver_source_fingerprint(frozen/'snapshot')
assert sha256(old)==receipt['binary_sha256']
model=root/'data/raw/benchmark/gen-ip002.mps.gz';out=root/'artifacts/exact-point-fixedwork';out.mkdir(exist_ok=True)
atomic_json(out/'experiment.json',dict(before=old_info,after=new_info,before_sha256=sha256(old),after_sha256=sha256(new),model_sha256=sha256(model),cpu=7,seconds=60,nodes=40000,memory_bytes=16*1024**3))
def restrict():
 os.sched_setaffinity(0,{7});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for rep in range(3):
 for side in (['before','after']if rep%2==0 else['after','before']):
  binary=old if side=='before'else new;d=out/f'{side}-{rep}';d.mkdir(exist_ok=True)
  command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'60','40000'];atomic_json(d/'command.json',command)
  started=time.monotonic()
  with (d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_BINARY_GRAPH='0'),preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
  r=json.loads((d/'result.json').read_text());atomic_json(d/'exit.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-started));assert p.returncode==0 and r['nodes']==37993 and r['status']=='numerical_error'
  print(json.dumps(dict(side=side,rep=rep,seconds=r['seconds'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'])),flush=True)
for rep in range(3):
 b=json.loads((out/f'before-{rep}/result.json').read_text());a=json.loads((out/f'after-{rep}/result.json').read_text())
 for key in ['status','objective','dual_bound','gap','nodes','values','lp_engine']:assert b[key]==a[key],key
 for key in ['lp_solves','lp_pivots','lp_refactors']:assert b[key]==a[key],key
print('all fixed-work semantic fields identical',flush=True)

for rep in range(3):
 for side in ['before','after']:
  d=out/f'{side}-{rep}'
  with (d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
  with (d/'proof.log').open('w')as stream:z=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(model),str(d/'result.json'),str(d/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
  assert q.returncode==z.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True and json.loads((d/'official-validation.json').read_text())['valid']is True
