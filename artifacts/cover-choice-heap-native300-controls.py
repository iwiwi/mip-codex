from pathlib import Path
import os,json,subprocess,time,resource,sys
root=Path.cwd();os.sched_setaffinity(0,{46});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_MEMORY_MIB='16384',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate
out=root/'artifacts/cover-choice-heap-native300-controls';out.mkdir();model=root/'data/raw/exploratory/z26.mps.gz';atomic_json(out/'experiment.json',dict(cpu=46,seconds=300,threads=1,memory_bytes=16*1024**3,input_sha256=sha256(model),split='exploratory; outside Benchmarkv2',purpose='fresh native comparison; no points or multipliers passed to Rust'));records=[]
for solver in ['highs','scip','glpk']:
 d=out/solver;d.mkdir();cmd=[str(root/'.venv/bin/python'),str(root/'scripts/comparison_worker.py'),solver,str(model),'300',str(d)];atomic_json(d/'job.json',dict(command=cmd,cpu=46));start=time.monotonic()
 with(d/'stdout.log').open('w')as output,(d/'stderr.log').open('w')as error:run=subprocess.run(cmd,stdout=output,stderr=error,timeout=420)
 atomic_json(d/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start));assert run.returncode==0;result=json.load(open(d/'result.json'));v=None
 if result.get('values')is not None:v=validate(model,result);atomic_json(d/'validation.json',v);assert v['valid']is True
 r=dict(solver=solver,**{k:result.get(k)for k in ['status','objective','dual_bound','seconds','nodes','version']},gmp_valid=None if v is None else v['valid']);records.append(r);atomic_json(out/'records.json',records);print(json.dumps(r),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=len(records)))
