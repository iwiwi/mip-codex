#!/usr/bin/env python3
"""Run all solvers with identical external wall/memory/CPU limits.
Each result, log, incumbent and independent check survives interruption.
"""
import argparse,datetime,gzip,hashlib,json,os,pathlib,re,resource,signal,subprocess,sys,time
ROOT=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('--seconds',type=float,default=10);p.add_argument('--count',type=int,default=3);p.add_argument('--output',required=True);p.add_argument('--instances',nargs='*');p.add_argument('--solvers',nargs='+',default=['rust','scip','highs','glpk']);p.add_argument('--cpu',type=int,default=min(os.sched_getaffinity(0)));args=p.parse_args()
out=pathlib.Path(args.output).resolve();out.mkdir(parents=True,exist_ok=True)
assert not (out/'run.json').exists(), 'Choose a fresh output directory to preserve prior results'
manifest=json.loads((ROOT/'docs/benchmark-manifest.json').read_text())['instances']
if args.instances:
 lookup={r['name']:r for r in manifest};chosen=[lookup[n] for n in args.instances]
 assert all(r['split']=='development' for r in chosen),'Use development instances for improvement decisions'
else:chosen=sorted([r for r in manifest if r['split']=='development'],key=lambda r:(ROOT/r['path']).stat().st_size)[:args.count]
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',RAYON_NUM_THREADS='1')
metadata=dict(started=datetime.datetime.now(datetime.timezone.utc).isoformat(),seconds=args.seconds,outer_grace_seconds=5,memory_bytes=16*1024**3,cpu=args.cpu,instances=[r['name'] for r in chosen],solvers=args.solvers,command=sys.argv)
metadata['rust_binary_sha256']=hashlib.sha256((ROOT/'target/release/mip-codex').read_bytes()).hexdigest()
metadata['source_sha256']={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in (ROOT/'src').glob('*.rs')}
metadata['git_head']=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip()
(out/'run.json').write_text(json.dumps(metadata,indent=2)+'\n')
def limits():
 os.sched_setaffinity(0,{args.cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for row in chosen:
 mps=out/(row['name']+'.mps');mps.write_bytes(gzip.decompress((ROOT/row['path']).read_bytes()))
 for solver in args.solvers:
  prefix=out/(row['name']+'.'+solver);resultpath=pathlib.Path(str(prefix)+'.solution.json');logpath=pathlib.Path(str(prefix)+'.log');timepath=pathlib.Path(str(prefix)+'.time')
  command=[str(ROOT/'target/release/mip-codex'),'solve',str(mps),str(args.seconds)] if solver=='rust' else [str(ROOT/'.venv/bin/python'),str(ROOT/'scripts/solver_worker.py'),solver,str(mps),str(args.seconds),str(resultpath)]
  start=time.monotonic();status='finished';code=None
  with logpath.open('w') as log:
   try:
    with resultpath.open('w') if solver=='rust' else open(os.devnull,'w') as output:
     run=subprocess.Popen(['/usr/bin/time','-v','-o',str(timepath),*command],stdout=output,stderr=log,preexec_fn=limits,env=env,start_new_session=True)
     try:code=run.wait(timeout=args.seconds+5)
     except subprocess.TimeoutExpired:
      os.killpg(run.pid,signal.SIGKILL);run.wait();raise
   except subprocess.TimeoutExpired:status='external_timeout'
  record=dict(instance=row['name'],solver=solver,termination=status,exit_code=code,wall_seconds=time.monotonic()-start,solution_file=resultpath.name)
  if code not in (None,0):record['termination']='process_failure'
  if timepath.exists():
   match=re.search(r'Maximum resident set size \(kbytes\): (\d+)',timepath.read_text())
   record['max_rss_kib']=int(match[1]) if match else None
  if resultpath.exists() and resultpath.stat().st_size:
   try:result=json.loads(resultpath.read_text())
   except json.JSONDecodeError:record['termination']='invalid_output';result={}
   record['result']={k:v for k,v in result.items() if k not in ('values','variable_names')}
   primal,dual=result.get('objective'),result.get('dual_bound')
   record['gap']=abs(primal-dual)/max(1,abs(primal),abs(dual)) if primal is not None and dual is not None else None
   if result.get('values') is not None:
    check=subprocess.run([str(ROOT/'.venv/bin/python'),str(ROOT/'scripts/check_solution.py'),str(mps),str(resultpath)],capture_output=True,text=True,env=env)
    pathlib.Path(str(prefix)+'.check.log').write_text(check.stdout+check.stderr);record['independent_check_passed']=check.returncode==0
    if check.returncode:record['termination']='invalid_solution'
  pathlib.Path(str(prefix)+'.result.json').write_text(json.dumps(record,indent=2)+'\n')
  print(json.dumps(record),flush=True)
 mps.unlink()
metadata['finished']=datetime.datetime.now(datetime.timezone.utc).isoformat();(out/'run.json').write_text(json.dumps(metadata,indent=2)+'\n')
