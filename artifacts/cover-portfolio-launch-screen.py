from pathlib import Path
import os,json,subprocess,sys
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/cover-portfolio-development-screen';out.mkdir();origin=json.load(open('artifacts/cover-portfolio-origin.json'));binary=root/'artifacts/cover-portfolio-frozen/bin/mip-codex';record=dict(path=str(binary),sha256=sha256(binary),build_info=origin['build_info']);atomic_json(out/'experiment.json',dict(shards=5,binaries={k:record for k in ['before','after']},seconds=10,threads=1,memory_bytes=16*1024**3,cpus=[31,32,33,34,35],cover_restarts={'before':0,'after':1},purpose='development188 only; same frozen binary graph cover portfolio off/on; default remains off'))
workers=[]
for shard,cpu in enumerate([31,32,33,34,35]):
 with (out/f'worker-{shard}.log').open('w')as log:p=subprocess.Popen([sys.executable,'artifacts/cover-portfolio-development-screen.py',str(cpu),str(shard),'5'],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(shard=shard,cpu=cpu,pid=p.pid))
atomic_json(out/'workers.json',workers);print(json.dumps(workers))
