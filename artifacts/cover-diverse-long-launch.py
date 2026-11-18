from pathlib import Path
import json,hashlib,subprocess
root=Path.cwd();jobs=json.load(open('artifacts/cover-neutral-v1/z26/audit-jobs.json'));points=[]
for r in jobs:
 p=Path(r['point']);assert hashlib.sha256(p.read_bytes()).hexdigest()==r['sha256'];v=json.load(open(p));assert v['objective']==-1197;points.append(set(n for n,x in v['values'].items()if x==1))
chosen=[0]
while len(chosen)<3:
 nxt=max((i for i in range(len(points))if i not in chosen),key=lambda i:(min(len(points[i]^points[j])for j in chosen),-i));chosen.append(nxt)
out=root/'artifacts/cover-diverse-long-v1';out.mkdir();spec=dict(selection='own neutral parents only; parent0 then greedy farthest minimum symmetric difference; no known/native solution input',indices=chosen,pairwise_distances=[[len(points[i]^points[j])for j in chosen]for i in chosen],source_parent_manifest_sha256=hashlib.sha256(Path('artifacts/cover-neutral-v1/z26/audit-jobs.json').read_bytes()).hexdigest(),private_only=True);(out/'experiment.json').write_text(json.dumps(spec,indent=2));workers=[]
for k,(i,cpu)in enumerate(zip(chosen,[40,47,50])):
 log=out/f'worker-{k}.log'
 with log.open('w')as f:p=subprocess.Popen([str(root/'.venv/bin/python'),'artifacts/cover-diverse-long-worker.py',str(cpu),jobs[i]['point'],str(out/str(k))],stdout=f,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(index=k,parent=i,cpu=cpu,pid=p.pid,log=str(log)))
(out/'workers.json').write_text(json.dumps(workers,indent=2));print(json.dumps(workers),flush=True)
