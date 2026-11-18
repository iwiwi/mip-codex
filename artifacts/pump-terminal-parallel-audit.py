from pathlib import Path
import os,sys,json,subprocess,resource,signal,time,hashlib,concurrent.futures
cpus=list(map(int,os.environ.get('MIP_AUDIT_CPUS','41,42,43,44').split(',')));assert len(cpus)==4 and len(set(cpus))==4
root=Path.cwd();sys.path.insert(0,str(root/'scripts'));from benchmark_common import atomic_json,sha256
prefix=sys.argv[1];assert prefix in ['pump-terminal']
source=root/'artifacts'/(prefix+'-development-screen');prior=root/'artifacts'/(prefix+'-development-audit');out=root/'artifacts'/(prefix+'-development-parallel-audit');out.mkdir();frozen=root/'artifacts/pump-terminal-frozen';assert sha256(frozen/'scripts/verify_cover_cuts.py')==sha256(root/'scripts/verify_cover_cuts.py')
manifest={r['name']:r for r in json.load(open(root/'data/manifest.json'))['instances']};old=[];records={};jobs=[]
for r in old:
 key=(r['name'],r['variant']);p=source/'cases'/key[0]/key[1]/'result.json';model=root/manifest[key[0]]['path'];assert manifest[key[0]]['split']=='development';assert r['input_sha256']==sha256(model)and r['result_sha256']==sha256(p);assert r['cut_validation']['valid']is True
 has=json.loads(p.read_text()).get('values')is not None
 if has:assert r['strict']['valid']is True and r['official']['valid']is True
 else:assert r['strict']is None and r['official']is None
 assert key not in records;records[key]=r
atomic_json(out/'reused-records.json',list(records.values()));atomic_json(out/'experiment.json',dict(prefix=prefix,reused=len(records),cpus=cpus,threads_per_worker=1,memory_per_worker_bytes=16*1024**3,checker_sha256=sha256(frozen/'scripts/verify_cover_cuts.py'),driver_sha256=sha256(Path(__file__)),prior=str(prior),source=str(source)))
for m in manifest.values():
 if m['split']!='development':continue
 for variant in ['before','after']:
  if(m['name'],variant)not in records:jobs.append(dict(name=m['name'],variant=variant))
atomic_json(out/'jobs.json',jobs)
def worker(shard):
 os.sched_setaffinity(0,{cpus[shard]});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,str(frozen/'scripts'));from verify_cover_cuts import validate_cut_proofs
 done=[]
 try:
  for i,job in enumerate(jobs):
   if i%4!=shard:continue
   name=job['name'];variant=job['variant'];m=manifest[name];model=root/m['path'];p=source/'cases'/name/variant/'result.json'
   while not p.exists():
    exits=list(source.glob('worker-*-exit.json'))
    if len(exits)==5:raise ValueError('terminal screen has missing result')
    time.sleep(1)
   r=json.loads(p.read_text());d=out/name/variant;d.mkdir(parents=True);assert sha256(model)==m['sha256'];record=dict(name=name,variant=variant,solver_status=r['status'],input_sha256=m['sha256'],result_sha256=sha256(p),cut_validation=validate_cut_proofs(model,r),strict=None,official=None)
   assert record['cut_validation']['valid']is True,record
   if r.get('values')is not None:
    cmd=[str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(p),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(d)]
    with(d/'audit.log').open('w')as stream:
     run=subprocess.run(cmd,stdout=stream,stderr=subprocess.STDOUT,timeout=600);assert run.returncode==0
    for kind in ['strict','official']:
     record[kind]=json.loads((d/(kind+'-validation.json')).read_text());assert record[kind]['valid']is True
   atomic_json(d/'audit-record.json',record);done.append(record);atomic_json(out/f'worker-{shard}-progress.json',dict(completed=len(done)));print(prefix,shard,len(done),name,variant,flush=True)
  atomic_json(out/f'worker-{shard}-exit.json',dict(status='complete',completed=len(done)));return done
 except BaseException as e:atomic_json(out/f'worker-{shard}-exit.json',dict(status='failed',completed=len(done),error=f'{type(e).__name__}: {e}'));raise
# Fork workers only after immutable reused records and jobs have been written.
import multiprocessing
with concurrent.futures.ProcessPoolExecutor(max_workers=4,mp_context=multiprocessing.get_context('fork'))as pool:
 for done in pool.map(worker,range(4)):
  for r in done:
   key=(r['name'],r['variant']);assert key not in records;records[key]=r
assert len(records)==376
values=list(records.values());points=sum(r['strict']is not None for r in values);atomic_json(out/'records.json',values);atomic_json(out/'summary.json',dict(completed=376,cut_valid=376,cut_invalid=0,points=points,strict_valid=points,strict_invalid=0,strict_unverified=0,official_valid=points,reused=len(old),fresh=376-len(old)));atomic_json(out/'exit.json',dict(status='complete'));print(prefix,'complete',points,flush=True)
