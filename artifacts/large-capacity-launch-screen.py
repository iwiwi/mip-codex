from pathlib import Path
import os,json,subprocess,sys
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/large-capacity-development-screen';out.mkdir();origin=json.load(open('artifacts/large-capacity-origin.json'));binary=root/'artifacts/large-capacity-frozen/bin/mip-codex';record=dict(path=str(binary),sha256=sha256(binary),build_info=origin['build_info']);atomic_json(out/'experiment.json',dict(shards=5,binaries={k:record for k in ['before','after']},seconds=10,threads=1,memory_bytes=16*1024**3,cpus=[31,32,33,34,35],large_capacity={'before':'explicit0','after':'explicit1'},purpose='development188 only; same frozen binary capacity off/on; candidate defaults off; all other main126 flags unchanged'))
workers=[]
for shard,cpu in enumerate([31,32,33,34,35]):
 with (out/f'worker-{shard}.log').open('w')as log:p=subprocess.Popen([sys.executable,'artifacts/large-capacity-development-screen.py',str(cpu),str(shard),'5'],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(shard=shard,cpu=cpu,pid=p.pid))
atomic_json(out/'workers.json',workers);print(json.dumps(workers))
