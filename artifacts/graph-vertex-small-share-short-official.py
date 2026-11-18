from pathlib import Path
import os,subprocess,json,sys
os.sched_setaffinity(0,{44});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json
out=root/'artifacts/graph-vertex-small-share-short-official';out.mkdir();records=[]
for name in ['z26.mps.gz','cdc7-4-3-2.mps.gz']:
 for flag in [0,1]:
  source=root/'artifacts/graph-vertex-small-share-short-controls'/name/'60'/str(flag)/'result.json';d=out/name/str(flag);d.mkdir(parents=True)
  env=os.environ.copy();env['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute'
  with(d/'audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(root/'data/raw/exploratory'/name),str(source),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(d)],env=env,stdout=f,stderr=subprocess.STDOUT,check=True,timeout=600)
  for kind in ['strict','official']:assert json.load(open(d/f'{kind}-validation.json'))['valid']is True
  records.append(dict(name=name,flag=flag,strict_valid=True,official_valid=True));atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',outputs=4))
