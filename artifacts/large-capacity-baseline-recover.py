from pathlib import Path
import os,sys,json,subprocess,hashlib,datetime,collections,fcntl
root=Path.cwd();assert root==Path('/home/ubuntu/mip-codex');os.sched_setaffinity(0,{43});sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json,same_process
p=root/'artifacts/runs/baseline-v2-20261115';assert sha256(p/'config.json')=='39551a3b530d8c4e1ed4eb4069c361285e929a84a74d73966bb4c574b43c125c';c=json.load(open(p/'config.json'));identity=json.load(open(p/'coordinator-identity.json'));assert not same_process(identity)
for name,h in c['snapshot_sha256'].items():assert sha256(p/'snapshot'/name)==h
for name,h in c['library_sha256'].items():assert sha256(name)==h
summaries={str(f.relative_to(p)):sha256(f)for f in(p/'cases').glob('*/summary.json')};assert len(summaries)==719;active=[]
for j in c['jobs']:
 q=Path('/sys/fs/cgroup/mip-codex-bench')/c['id']/j['id'];ps=(q/'cgroup.procs').read_text().split()if q.exists()else[]
 if ps:
  d=p/'cases'/j['id'];alive=any((d/n).exists()and same_process(json.load(open(d/n)))for n in ['case-identity.json','supervisor-launch.json']);assert alive,'Do not launch while unowned live children remain';active.append(dict(job_id=j['id'],solver=j['solver'],processes=len(ps),identity_alive=alive,memory_events=(q/'memory.events').read_text(),commands=dict(collections.Counter(Path('/proc',pid,'comm').read_text().strip()for pid in ps))))
assert len(active)==1 and active[0]['solver']=='glpk';dest=root/'artifacts/large-capacity-baseline-recovery';dest.mkdir();
for name in ['coordinator-identity.json','coordinator-exit-code','coordinator.log','progress.json']:__import__('shutil').copy2(p/name,dest/name)
atomic_json(dest/'preflight.json',dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),prior_identity=identity,prior_alive=False,config_sha256=sha256(p/'config.json'),snapshot_files_verified=len(c['snapshot_sha256']),libraries_verified=len(c['library_sha256']),completed=719,active=active,no_space_in_prior_log='No space left on device'in(p/'coordinator.log').read_text(),summaries=summaries,action='restart frozen coordinator only; every terminal summary retained; live GLPK child identity must remain alive; no optimization restart'))
cmd=[str(root/'.venv/bin/python'),str(p/'snapshot/scripts/benchmark.py'),'run',str(p)];atomic_json(dest/'job.json',dict(command=cmd));
with(dest/'coordinator.log').open('w')as log:child=subprocess.Popen(cmd,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
atomic_json(dest/'launcher.json',dict(pid=child.pid,started_utc=datetime.datetime.now(datetime.timezone.utc).isoformat()));print(json.dumps(dict(pid=child.pid,completed=719,live_glpk_preserved=True)),flush=True)
