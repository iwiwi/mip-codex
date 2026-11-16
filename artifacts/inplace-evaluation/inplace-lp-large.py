import os,json,subprocess,time,sys
from pathlib import Path
cpu=int(sys.argv[1]);name=sys.argv[2];os.sched_setaffinity(0,{cpu});root=Path.cwd();old=root/'artifacts/runs/four-rounds-final-20261116/snapshot/bin/mip-codex';new=root/'target/release/mip-codex';env=os.environ.copy()
for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP']:env['MIP_'+f]='0'
records=[]
for i in range(2):
 for label,binary in [('before',old),('after',new)]:
  start=time.monotonic();run=subprocess.run([str(binary),'solve','data/raw/benchmark/'+name+'.mps.gz','60','1'],capture_output=True,text=True,env=env,check=True,timeout=70);r=json.loads(run.stdout);record=dict(pair=i,label=label,cpu=cpu,seconds_limit=60,memory_limit=None,wall_seconds=time.monotonic()-start,result=r);records.append(record);print(i,label,r['seconds'],r['dual_bound'],r['lp_pivots'],flush=True)
 Path('artifacts/inplace-lp-'+name+'.json').write_text(json.dumps(records,indent=2)+'\n')
