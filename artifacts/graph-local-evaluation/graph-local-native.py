import json,os,subprocess,resource,sys
from pathlib import Path
root=Path.cwd();os.sched_setaffinity(0,set(range(33,42)));os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1')
sys.path.insert(0,str(root/'scripts'))
from benchmark_common import sha256,atomic_json
model=root/'data/raw/exploratory/sorrell7.mps.gz';input_info=json.loads((root/'experiments/sorrell7-input-20261115.json').read_text());assert sha256(model)==input_info['sha256']
def restrict():
 os.sched_setaffinity(0,{7});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for solver in ['highs','scip','glpk']:
 d=root/'artifacts/graph-local-native'/solver;d.mkdir(parents=True,exist_ok=True)
 command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(root/'.venv/bin/python'),str(root/'scripts/comparison_worker.py'),solver,str(model),'15',str(d)]
 atomic_json(d/'experiment.json',dict(command=command,input=input_info,seconds=15,threads=1,memory_bytes=16*1024**3,cpu=7))
 with (d/'stdout.log').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0;r=json.loads((d/'result.json').read_text())
 if r.get('values')is not None:
  with (d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
  assert q.returncode==0;check=json.loads((d/'strict-validation.json').read_text())
 else:check=None
 print(json.dumps(dict(solver=solver,status=r['status'],objective=r.get('objective'),dual_bound=r.get('dual_bound'),strict=None if check is None else check['valid'])),flush=True)
