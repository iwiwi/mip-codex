import json,subprocess,os,resource
from pathlib import Path
root=Path.cwd();out=root/'artifacts/graph-polling-deadlines';out.mkdir(exist_ok=True)
def restrict():
 os.sched_setaffinity(0,{3});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
records=[]
for seconds in [0,.2,.5,1.]:
 d=out/str(seconds);d.mkdir(exist_ok=True)
 command=[str(root/'target/release/mip-codex'),'solve',str(root/'data/raw/benchmark/sorrell3.mps.gz'),str(seconds)]
 with (d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=dict(os.environ,MIP_BINARY_GRAPH='1'),preexec_fn=restrict,stdout=stream,stderr=err,timeout=10)
 r=json.loads((d/'result.json').read_text());assert p.returncode==0 and r['status']=='time_limit';assert r['seconds']<seconds+.25
 if seconds==0:assert r['values']is None and r['nodes']==0
 else:
  assert r['binary_graph']is not None
  with (d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(root/'data/raw/benchmark/sorrell3.mps.gz'),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
  with (d/'proof.log').open('w')as stream:z=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/verify_cover_cuts.py'),str(root/'data/raw/benchmark/sorrell3.mps.gz'),str(d/'result.json'),str(d/'proof.json')],stdout=stream,stderr=subprocess.STDOUT)
  assert q.returncode==z.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True and json.loads((d/'official-validation.json').read_text())['valid']is True
 record=dict(seconds=seconds,actual_seconds=r['seconds'],nodes=r['nodes'],status=r['status'],command=command)
 records.append(record);print(json.dumps(record),flush=True)
(out/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
