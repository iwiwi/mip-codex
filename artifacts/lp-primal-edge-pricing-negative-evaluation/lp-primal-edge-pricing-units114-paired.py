import os,json,subprocess,resource,signal,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();o=root/'artifacts/lp-primal-edge-pricing-units114-paired';o.mkdir();os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));model=root/'artifacts/lp-primal-edge-pricing-oracle/units/root-units-failure-114.mps';binary=root/'artifacts/lp-primal-edge-pricing-v1-diagnostic/bin/mip-codex';env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_PRIMAL_REPAIR='0',MIP_PROPAGATION='0',MIP_LP_ENGINE='revised')
for f in ['PARITY_EQUALITIES','THREE_ROW_CUTS','MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','OBJECTIVE_ROUNDING','STRONG_BRANCHING','LP_DIVING','PRIMAL_PROBE','RINS']:env['MIP_'+f]='0'
for label in ['before','after']:
 d=o/label;d.mkdir();e=dict(env,MIP_LP_EDGE_PRICING='0'if label=='before'else'1',MIP_ROOT_LP_DUMP=str(d/'root-lp.json'),MIP_LP_TRACE='1');cmd=[str(binary),'solve',str(model),'10','1'];atomic_json(d/'job.json',dict(command=cmd,environment=e,input_sha256=sha256(model),binary_sha256=sha256(binary)))
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:p=subprocess.run(cmd,stdout=out,stderr=err,env=e,timeout=70)
 assert p.returncode==0;r=json.loads((d/'result.json').read_text());print(label,r['status'],r['dual_bound'],r['lp_pivots'],r.get('lp_edge_pricing_updates'),flush=True)
atomic_json(o/'exit.json',dict(status='complete'))
