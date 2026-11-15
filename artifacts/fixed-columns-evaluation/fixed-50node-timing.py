import subprocess,json,os,time,hashlib
from pathlib import Path
root=Path('/home/ubuntu/mip-codex');out=root/'artifacts/fixed-50node-timing';out.mkdir(exist_ok=True)
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_PRIMAL_REPAIR='0',MIP_LP_DIVING='0',MIP_PRIMAL_PROBE='0',MIP_RINS='0',MIP_STRONG_BRANCHING='0',MIP_COVER_CUTS='0')
for rep in range(3):
 for side in (['before','after'] if rep%2==0 else ['after','before']):
  binary=root/('target/release/mip-codex' if side=='before' else 'artifacts/runs/root-fixed-columns-source/target/release/mip-codex')
  command=['taskset','-c','16',str(binary),'solve',str(root/'data/raw/benchmark/30n20b8.mps.gz'),'60','50']
  started=time.monotonic()
  with (out/f'{side}-{rep}.json').open('w')as stream,(out/f'{side}-{rep}.err').open('w')as err:p=subprocess.run(command,env=dict(env,MIP_FIXED_COLUMNS='1'if side=='after'else'0'),stdout=stream,stderr=err,timeout=75)
  r=json.loads((out/f'{side}-{rep}.json').read_text());record=dict(side=side,rep=rep,command=command,exit_code=p.returncode,wall=time.monotonic()-started,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],lp_solves=r['lp_solves'],seconds=r['seconds'],root_fixed_columns=r.get('root_fixed_columns'))
  (out/f'{side}-{rep}-meta.json').write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True)
