import ast,json,os,subprocess,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256
root=Path.cwd();os.sched_setaffinity(0,{5});snap=root/'artifacts/runs/fractional-cover-final-20261115';config=json.loads((snap/'config.json').read_text());verify_snapshot(snap,config)
frozen=snap/'snapshot/bin/mip-codex';current=root/'target/release/mip-codex'
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_LP_ENGINE='dense')
for flag in ['MIP_AGGREGATE_CUTS','MIP_PRIMAL_REPAIR','MIP_PROPAGATION','MIP_COVER_CUTS','MIP_ROUNDING_CUTS','MIP_OBJECTIVE_ROUNDING','MIP_STRONG_BRANCHING','MIP_LP_DIVING','MIP_PRIMAL_PROBE','MIP_RINS']:env[flag]='0'
records=[];out=root/'artifacts/aggregate-cuts-root-failure-control';out.mkdir(exist_ok=True)
for model in sorted(Path('artifacts/aggregate-cuts-final-root-units').glob('*failure*.mps')):
 pair={}
 for label,binary in [('frozen_pr56',frozen),('current',current)]:
  p=subprocess.run([str(binary),'solve',str(model),'10','1'],env=env,capture_output=True,text=True,timeout=20)
  (out/f'{model.stem}-{label}.json').write_text(p.stdout);(out/f'{model.stem}-{label}.err').write_text(p.stderr)
  r=json.loads(p.stdout);pair[label]={k:r[k]for k in ['status','objective','dual_bound','nodes']}
 assert pair['frozen_pr56']==pair['current'],pair
 records.append(dict(model=str(model),input_sha256=sha256(model),same=True,results=pair))
atomic_json(out/'results.json',records);print(json.dumps(dict(cases=len(records),same=sum(r['same']for r in records))))
