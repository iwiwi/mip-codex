from pathlib import Path
import os,sys,json,subprocess,datetime
root=Path.cwd();out=root/'artifacts/cover-vertex-penalty-v1';out.mkdir();(out/'experiment.json').write_text(json.dumps(dict(created_utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),pd_values=[1,5,10,20],cpus=[40,47,50,51],seconds_guard=1800,max_steps=100000000,watchdog=1920,threads=1,memory_bytes=16*1024**3,seed='same own Rust1198 parent0 for all configurations',selection='pd1 special perturbation plus fixed5/10/20, all before long outcomes',target=None,stop_value=None,production_changes=False),indent=2)+'\n');workers=[]
for pd,cpu in zip([1,5,10,20],[40,47,50,51]):
 with(out/f'worker-{pd}.log').open('w')as log:p=subprocess.Popen([sys.executable,'artifacts/cover-vertex-penalty-worker.py',str(cpu),str(pd)],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(pd=pd,cpu=cpu,pid=p.pid))
(out/'workers.json').write_text(json.dumps(workers,indent=2)+'\n');print(workers)
