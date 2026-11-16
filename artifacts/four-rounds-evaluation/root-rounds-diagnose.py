import json,os,resource,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
root=Path.cwd();name=sys.argv[1];cpu=int(sys.argv[2]);seconds=int(sys.argv[3]);rounds=list(map(int,sys.argv[4:]));os.sched_setaffinity(0,set(range(33,42)));snap=root/'artifacts/runs/clique-cuts-final-20261116';config=json.loads((snap/'config.json').read_text());verify_snapshot(snap,config);binary=snap/'snapshot/bin/mip-codex';record=next(r for r in config['instances']if r['name']==name+'.mps.gz');assert record['split']=='development';model=Path(record['absolute_path']);assert sha256(model)==record['sha256'];records=[]
def restrict():os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for rounds in rounds:
 d=root/f'artifacts/root-rounds-diagnostic{seconds}'/name/str(rounds);d.mkdir(parents=True,exist_ok=True);assert not(d/'experiment.json').exists();command=[str(binary),'solve',str(model),str(seconds)];env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_COVER_ROUNDS=str(rounds));atomic_json(d/'experiment.json',dict(command=command,rounds=rounds,cpu=cpu,threads=1,memory_bytes=16*1024**3,split='development',build_info=config['solver_build_info'],frozen_config_sha256=sha256(snap/'config.json'),binary_sha256=sha256(binary),input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')}))
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(command,stdout=out,stderr=err,env=env,preexec_fn=restrict,timeout=seconds+60)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0;r=json.loads((d/'result.json').read_text());proof=validate_cut_proofs(model,r);assert proof['valid'];atomic_json(d/'proof.json',proof)
 if r.get('values')is not None:
  with(d/'validation.log').open('w')as out:p=subprocess.run(['.venv/bin/python','scripts/benchmark_validate.py',str(model),str(d/'result.json'),'third_party/miplib-checker/bin/solchecker',str(d)],stdout=out,stderr=subprocess.STDOUT)
  assert p.returncode==0 and all(json.loads((d/f'{k}-validation.json').read_text())['valid']for k in ['strict','official'])
 record=dict(name=name,rounds=rounds,**{k:r.get(k)for k in ['status','objective','dual_bound','nodes','first_primal_seconds','cut_rounds','cut_lp_solves','cut_seconds','cut_stats']});records.append(record);print(json.dumps(record),flush=True)
atomic_json(root/f'artifacts/root-rounds-{name}-{seconds}.json',records)
