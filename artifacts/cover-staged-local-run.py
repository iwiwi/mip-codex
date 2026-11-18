from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
cpu=int(sys.argv[1]);name=sys.argv[2];root=Path.cwd();os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/cover-staged-local-v1'/name;out.mkdir();model=root/'data/raw/exploratory'/name;weighted=json.load(open('artifacts/cover-heap-fixed-work/experiment.json'))['binaries']['heap'];wb=Path(weighted['path']);assert sha256(wb)==weighted['sha256'];vb=root/'artifacts/cover-vertex-penalty-smoke/bin/profile';assert sha256(vb)==json.load(open('artifacts/cover-vertex-penalty-smoke/source.json'))['profile_sha256'];records=[]
for label in ['initial']+(['weighted','vertex']if name=='z26.mps.gz'else['vertex','weighted']):
 d=out/label;d.mkdir();seed=out/'initial/result.json';env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_DIAGNOSTIC_COVER_INITIAL='100000000',MIP_DIAGNOSTIC_COVER_EXTENDED='100000000');cmd=([str(wb),str(model),'15','single',str(d/'result.json')]+([]if label=='initial'else[str(seed)]))if label!='vertex'else[str(vb),str(model),'15','1','149372736029117923','100000000',str(d/'result.json'),str(seed)];atomic_json(d/'job.json',dict(command=cmd,cpu=cpu,input_sha256=sha256(model),seed=None if label=='initial'else str(seed),seed_sha256=None if label=='initial'else sha256(seed),memory_bytes=16*1024**3,threads=1,seconds=15,binary_sha256=sha256(wb if label!='vertex'else vb),target=None,stop_value=None));start=time.monotonic()
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,stdout=so,stderr=se,env=env,start_new_session=True);atomic_json(d/'process.json',dict(pid=p.pid,process_group=p.pid,started_unix=time.time()));watchdog=False
  try:p.wait(timeout=75)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start));assert p.returncode==0 and not watchdog
 r=json.load(open(d/'result.json'));a=d/'audit';a.mkdir()
 with(a/'audit.log').open('w')as log:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(a)],stdout=log,stderr=subprocess.STDOUT,timeout=600,check=True)
 for kind in ['strict','official']:assert json.load(open(a/(kind+'-validation.json')))['valid']
 record=dict(label=label,objective=r['objective'],seconds=r['component_seconds'],gmp_valid=True,official_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(records=records,point_gmp_official_valid=3,private_only=True,production_changes=False,full_solver_same_time_claim=False))
