import json,os,sys,subprocess,resource
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import sha256,require_solver_origin,atomic_json
os.sched_setaffinity(0,set(range(33,42)));os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1')
enabled=int(sys.argv[1]);core=int(sys.argv[2]);side='after'if enabled else'before';binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
def restrict():
 os.sched_setaffinity(0,{core});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name,path,split in [('sorrell3','data/raw/benchmark/sorrell3.mps.gz','development'),('sorrell7','data/raw/exploratory/sorrell7.mps.gz','exploration_outside_benchmark_v2')]:
 model=root/path;d=root/'artifacts/graph-prefix-pairs'/side/name;d.mkdir(parents=True,exist_ok=True)
 command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'15'];atomic_json(d/'experiment.json',dict(command=command,build_info=info,binary_sha256=sha256(binary),input_sha256=sha256(model),split=split,root_prefix=bool(enabled),cpu=core,threads=1,memory_bytes=16*1024**3))
 with (d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,MIP_BINARY_GRAPH='1',MIP_GRAPH_ROOT_COLORING='1',MIP_GRAPH_ROOT_PREFIX=str(enabled),MIP_GRAPH_LOCAL_SEARCH='1'),preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0
 with (d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
 with (d/'proof.log').open('w')as stream:z=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(model),str(d/'result.json'),str(d/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
 r=json.loads((d/'result.json').read_text());print(json.dumps(dict(side=side,name=name,objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],graph={k:r['binary_graph'][k]for k in ['local','root_upper','remaining_upper']})),flush=True)
 assert q.returncode==z.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True and json.loads((d/'official-validation.json').read_text())['valid']is True
 if enabled:assert r['binary_graph']['local']is not None and r['binary_graph']['local']['validation_failures']==0
