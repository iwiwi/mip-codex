import subprocess,os
from pathlib import Path
env=['env']
for f in ['MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP']:env.append('MIP_'+f+'=0')
for label,binary in [('before','artifacts/runs/lp-inplace-final-20261116/snapshot/bin/mip-codex'),('after','target/release/mip-codex')]:
 cmd=env+['taskset','-c','3',binary,'solve','data/raw/benchmark/binkar10_1.mps.gz','5','1']
 with open('artifacts/mir-inplace-perf-'+label+'-stdout.json','w')as out:
  subprocess.run(['sudo','-n','perf','stat','-x,','-r','8','-e','cycles,instructions,cache-references,cache-misses,branches,branch-misses','-o','artifacts/mir-inplace-perf-'+label+'.csv','--']+cmd,stdout=out,check=True)
 with open('artifacts/mir-inplace-profile-'+label+'-stdout.json','w')as out:
  subprocess.run(['sudo','-n','perf','record','-F','997','-e','cycles:u','-o','artifacts/mir-inplace-profile-'+label+'.data','--']+cmd,stdout=out,check=True)
 with open('artifacts/mir-inplace-profile-'+label+'.txt','w')as out:subprocess.run(['sudo','-n','perf','report','--stdio','--no-children','-i','artifacts/mir-inplace-profile-'+label+'.data'],stdout=out,check=True)
