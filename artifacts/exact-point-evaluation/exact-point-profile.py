import json,os,sys,resource,subprocess
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import atomic_json,sha256,require_solver_origin
os.sched_setaffinity(0,set(range(33,42)));d=root/'artifacts/exact-point-profile';d.mkdir(exist_ok=True);binary=root/'target/release/mip-codex';model=root/'data/raw/benchmark/gen-ip002.mps.gz'
command=['/usr/bin/time','-v','-o',str(d/'resource.log'),'sudo','perf','record','-F','99','-e','cpu-clock','-o',str(d/'perf.data'),'--',str(binary),'solve',str(model),'5']
def restrict():
 os.sched_setaffinity(0,{7});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
atomic_json(d/'experiment.json',dict(build_info=require_solver_origin(root,binary),source_git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),binary_sha256=sha256(binary),input_sha256=sha256(model),split='development',cpu=7,threads=1,memory_bytes=16*1024**3,command=command,kind='profile_not_performance_comparison'))
with (d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(command,preexec_fn=restrict,env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1'),stdout=out,stderr=err,timeout=70)
atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0
subprocess.run(['sudo','chown',f'{os.getuid()}:{os.getgid()}',str(d/'perf.data')],check=True)
with (d/'report.txt').open('w')as out:subprocess.run(['perf','report','--stdio','--no-children','-i',str(d/'perf.data')],stdout=out,check=True)
r=json.loads((d/'result.json').read_text())
if r.get('values')is not None:
 with (d/'validation.log').open('w')as out:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=out,stderr=subprocess.STDOUT)
 assert q.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True
with(d/'proof.log').open('w')as out:z=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(model),str(d/'result.json'),str(d/'proof.json')],stdout=out,stderr=subprocess.STDOUT)
assert z.returncode==0
print(json.dumps(dict(status=r['status'],objective=r.get('objective'),dual=r.get('dual_bound'),nodes=r['nodes'])))
