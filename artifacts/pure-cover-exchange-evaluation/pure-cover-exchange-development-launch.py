import subprocess,time,json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json
root=Path.cwd();out=root/'artifacts/pure-cover-exchange-development-screen';jobs=[];started=set()
while len(started)<5:
 for shard,cpu in enumerate(range(3,8)):
  if shard in started:continue
  prior=root/f'artifacts/dual-refactor-retry-development-screen/worker-{shard}-exit.json'
  if not prior.exists():continue
  assert json.load(open(prior))['status']=='complete'
  log=(out/f'worker-{shard}-launcher.log').open('w');p=subprocess.Popen([sys.executable,'artifacts/pure-cover-exchange-development-screen.py',str(cpu),str(shard),'5'],stdout=log,stderr=subprocess.STDOUT,start_new_session=True);jobs.append((p,log,shard));started.add(shard);atomic_json(out/f'worker-{shard}-launcher.json',dict(pid=p.pid,cpu=cpu,prior_exit=str(prior)));print(shard,cpu,p.pid,flush=True)
 time.sleep(2)
for p,log,shard in jobs:
 p.wait();log.close();assert p.returncode==0,(shard,p.returncode)
atomic_json(out/'launcher-exit.json',dict(status='complete'))
