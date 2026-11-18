from pathlib import Path
import os,sys,json,subprocess
os.sched_setaffinity(0,{45});root=Path.cwd();os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,'scripts');from benchmark_common import atomic_json
out=root/'artifacts/graph-vertex-hour-official';out.mkdir();assert json.load(open('artifacts/graph-vertex-hour-controls/exit.json'))['status']=='complete';records=[]
for flag in [0,1]:
 d=out/str(flag);d.mkdir();model=root/'data/raw/exploratory/z26.mps.gz';result=root/'artifacts/graph-vertex-hour-controls'/str(flag)/'result.json'
 with(d/'audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(result),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(d)],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=600)
 for kind in ['strict','official']:assert json.load(open(d/f'{kind}-validation.json'))['valid']is True
 r=json.load(open(result));stats=(r.get('binary_graph')or{}).get('vertex_local');records.append(dict(flag=flag,objective=r['objective'],dual_bound=r['dual_bound'],vertex=stats,strict_valid=True,official_valid=True));atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',outputs=2,strict_official_valid=2,optimizer_not_rerun=True));print(json.dumps(records),flush=True)
