import json,os,resource,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,sha256,atomic_json
from benchmark import verify_snapshot
from verify_cover_cuts import validate_cut_proofs
root=Path.cwd();enabled=int(sys.argv[1]);cpu=int(sys.argv[2]);seconds=int(sys.argv[3]);side='after'if enabled else'before';os.sched_setaffinity(0,set(range(33,42)))
if enabled:binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
else:
 snap=root/'artifacts/runs/root-refinement-final-20261115';c=json.loads((snap/'config.json').read_text());verify_snapshot(snap,c);binary=snap/'snapshot/bin/mip-codex';info=json.loads(subprocess.check_output([str(binary),'build-info'],text=True));assert info==c['solver_build_info']
def restrict():
 os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in ['germanrr','glass4','app1-1']:
 model=root/'data/raw/benchmark'/(name+'.mps.gz');d=root/f'artifacts/zero-restoration-main{seconds}'/side/name;d.mkdir(parents=True,exist_ok=True)
 command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),str(seconds)]
 atomic_json(d/'experiment.json',dict(command=command,split='development',input_sha256=sha256(model),build_info=info,binary_sha256=sha256(binary),zero_objective_restoration=bool(enabled),cpu=cpu,threads=1,memory_bytes=16*1024**3))
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_AGGREGATE_CUTS='1'),preexec_fn=restrict,stdout=out,stderr=err,timeout=seconds+60)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0
 r=json.loads((d/'result.json').read_text());proof=validate_cut_proofs(model,r);atomic_json(d/'proof.json',proof);assert proof['valid']
 if r.get('values')is not None:
  with(d/'validation.log').open('w')as out:q=subprocess.run(['.venv/bin/python','scripts/benchmark_validate.py',str(model),str(d/'result.json'),'third_party/miplib-checker/bin/solchecker',str(d)],stdout=out,stderr=subprocess.STDOUT)
  assert q.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']
 print(json.dumps(dict(side=side,name=name,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],refinement={k:r.get(k)for k in ['lp_refinement_attempts','lp_refinement_steps','lp_refinement_successes','lp_restoration_attempts','lp_restoration_successes','lp_restoration_infeasible_candidates','lp_restoration_pivots']})),flush=True)
