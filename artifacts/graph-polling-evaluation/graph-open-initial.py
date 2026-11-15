import json,os,subprocess,sys,resource
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import sha256,atomic_json,require_solver_origin
receipt=json.loads((root/'experiments/binary-graph-snapshot-20261115.json').read_text());model=root/'data/raw/exploratory/sorrell7.mps.gz';input_info=json.loads((root/'experiments/sorrell7-input-20261115.json').read_text());assert sha256(model)==input_info['sha256']
out=root/'artifacts/graph-open-initial';out.mkdir(exist_ok=True)
def restrict():
 os.sched_setaffinity(0,{1});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for side,binary in [('before',Path(receipt['directory'])/'snapshot/bin/mip-codex'),('after',root/'target/release/mip-codex')]:
 d=out/side;d.mkdir(exist_ok=True);command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'15']
 atomic_json(d/'experiment.json',dict(command=command,binary_sha256=sha256(binary),input=input_info,seconds=15,cpu=1,threads=1,memory_bytes=16*1024**3))
 with (d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,MIP_BINARY_GRAPH='1',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1'),preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0
 with (d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
 with (d/'proof.log').open('w')as stream:z=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(model),str(d/'result.json'),str(d/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
 r=json.loads((d/'result.json').read_text());print(json.dumps(dict(side=side,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],strict=json.loads((d/'strict-validation.json').read_text())['valid'],proof_exit=z.returncode)),flush=True)
 assert q.returncode==0 and z.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True and json.loads((d/'official-validation.json').read_text())['valid']is True
