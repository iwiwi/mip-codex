from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource,signal
root=Path.cwd();os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/pump-candidate-descent-v1';out.mkdir();(out/'bin').mkdir();binary=out/'bin/profile';shutil.copy2(root/'target/release/examples/pump_candidate_descent',binary);source=root/'artifacts/pump-terminal-pilot';manifest={r['name']:r for r in json.load(open(source/'jobs.json'))};analysis=json.load(open(source/'analysis.json'));jobs=[]
for r in analysis['rows']:
 if r['after']['objective']is None:continue
 name=r['name'];p=source/'cases'/name/'1/result.json';d=p.parent/'audit';assert all(json.load(open(d/(k+'-validation.json')))['valid']for k in ['strict','official']);jobs.append(dict(name=name,point=str(p),point_sha256=sha256(p),input=manifest[name]['path'],input_sha256=manifest[name]['sha256'],before=r['after']['objective']))
assert len(jobs)==6;atomic_json(out/'jobs.json',jobs);atomic_json(out/'experiment.json',dict(cpu=45,seconds=0.1,memory_bytes=16*1024**3,threads=1,selection='all6 pilot ON feasible outputs, before descent outcomes',binary_sha256=sha256(binary),source_sha256=sha256(root/'examples/pump_candidate_descent.rs'),base_source_fingerprint='8f4338607ff37aab',seed_validation_reused_same_input_point_sha256=6,private_only=True));records=[]
for job in jobs:
 d=out/job['name'];d.mkdir();model=root/job['input'];point=Path(job['point']);assert sha256(model)==job['input_sha256']and sha256(point)==job['point_sha256'];cmd=[str(binary),str(model),str(point),'0.1',str(d/'result.json')];atomic_json(d/'job.json',dict(command=cmd));began=time.monotonic()
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,start_new_session=True);watchdog=False
  try:p.wait(timeout=30)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-began));assert p.returncode==0 and not watchdog
 r=json.load(open(d/'result.json'));a=d/'audit';a.mkdir()
 with(a/'audit.log').open('w')as log:q=subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(a)],stdout=log,stderr=subprocess.STDOUT,timeout=600)
 vals={k:json.load(open(a/(k+'-validation.json')))for k in ['strict','official']};record=dict(name=job['name'],before=job['before'],after=r['objective'],seconds=r['component_seconds'],stats=r['stats'],gmp_valid=vals['strict']['valid'],official_valid=vals['official']['valid'],validator_exit_code=q.returncode);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=len(records),gmp_valid=sum(r['gmp_valid']is True for r in records),official_valid=sum(r['official_valid']is True for r in records),improved=sum(r['after']<r['before'] for r in records),seed_validations_reused=6,production_changes=False,new_record=False))
