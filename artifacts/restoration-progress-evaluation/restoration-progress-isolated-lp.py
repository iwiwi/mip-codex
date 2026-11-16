import os,json,subprocess,time,statistics
from pathlib import Path
os.sched_setaffinity(0,{40});root=Path.cwd();old=root/'artifacts/runs/dual-restoration-final-20261116/snapshot/bin/mip-codex';new=root/'artifacts/restoration-progress-final-diagnostic-20261116/bin/mip-codex';env=os.environ.copy()
for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP']:env['MIP_'+f]='0'
records=[]
for i in range(12):
 for label,binary in ([('before',old),('after',new)] if i%2==0 else [('after',new),('before',old)]):
  start=time.monotonic();run=subprocess.run([str(binary),'solve','data/raw/benchmark/binkar10_1.mps.gz','5','1'],capture_output=True,text=True,env=env,check=True,timeout=20);r=json.loads(run.stdout);record=dict(pair=i,label=label,wall_seconds=time.monotonic()-start,result=r);records.append(record);print(i,label,r['seconds'],r['dual_bound'],flush=True)
Path('artifacts/restoration-progress-isolated-lp.json').write_text(json.dumps(records,indent=2)+'\n')
