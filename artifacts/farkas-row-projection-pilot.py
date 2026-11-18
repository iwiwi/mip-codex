from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
root=Path.cwd();os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/farkas-row-projection-pilot';out.mkdir();binary=root/'artifacts/farkas-row-projection-frozen/bin/mip-codex';origin=json.load(open('artifacts/farkas-row-projection-origin.json'));assert sha256(binary)==origin['binary_sha256'];manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};atomic_json(out/'experiment.json',dict(origin=origin,cpu=40,threads=1,memory_bytes=16*1024**3,seconds=10,trace_both=True,default_off=True,point_transfer=False));records=[]
for case,name in enumerate(['neos-3754480-nidda.mps.gz','neos-2657525-crna.mps.gz','glass4.mps.gz','leo1.mps.gz']):
 m=manifest[name];assert m['split']=='development';model=root/m['path'];assert sha256(model)==m['sha256']
 for flag in ([0,1]if case%2==0 else[1,0]):
  d=out/name/str(flag);d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_FARKAS_ROW_PROJECT=str(flag),MIP_LP_TRACE='1');cmd=[str(binary),'solve',str(model),'10'];atomic_json(d/'job.json',dict(command=cmd,input_sha256=m['sha256'],cpu=40,environment={k:v for k,v in env.items()if k.startswith('MIP_')}));start=time.monotonic();watchdog=False
  with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
   p=subprocess.Popen(cmd,env=env,stdout=so,stderr=se,start_new_session=True)
   try:p.wait(timeout=70)
   except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
  atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start));assert p.returncode==0 and not watchdog;r=json.load(open(d/'stdout.log'));atomic_json(d/'result.json',r);proof=validate_cut_proofs(model,r);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True;point=None
  if r.get('values')is not None:
   audit=d/'point-audit';audit.mkdir()
   with(audit/'audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(audit)],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=600)
   for kind in ['strict','official']:assert json.load(open(audit/f'{kind}-validation.json'))['valid']is True
   point=True
  record=dict(name=name,flag=flag,**{k:r.get(k)for k in ['status','objective','dual_bound','nodes','seconds','lp_solves','lp_pivots','lp_farkas_row_projection_trials','lp_farkas_row_projections','infeasibility_proof_failures','lp_search_termination']},point_valid=point,proof_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=8,proof_valid=8,gmp_official_valid=sum(x['point_valid']is True for x in records)))
