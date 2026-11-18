from pathlib import Path
import os,sys,json,subprocess,time,resource,signal
root=Path.cwd();os.sched_setaffinity(0,{43});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate;from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/numerical-cold-parent-trace';out.mkdir();binary=Path('/home/ubuntu/mip-codex/artifacts/cover-long-round-main-primary-audit/bin/mip-codex');assert sha256(binary)=='0bf00779798a657b02581a960261473b3093a21aae08d206b0d180f328a19876';manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for name in ['neos-3754480-nidda.mps.gz','neos-2657525-crna.mps.gz','glass4.mps.gz','leo1.mps.gz']:
 m=manifest[name];assert m['split']=='development';model=root/m['path'];assert sha256(model)==m['sha256'];d=out/name;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_TRACE='1');cmd=[str(binary),'solve',str(model),'10'];atomic_json(d/'job.json',dict(command=cmd,binary_sha256=sha256(binary),input_sha256=m['sha256'],cpu=43,threads=1,memory_bytes=16*1024**3,environment=dict(MIP_LP_TRACE='1'),diagnostic_trace=True));start=time.monotonic();watchdog=False
 with(d/'stdout.log').open('w')as so,(d/'stderr.log').open('w')as se:
  p=subprocess.Popen(cmd,env=env,stdout=so,stderr=se,start_new_session=True)
  try:p.wait(timeout=70)
  except subprocess.TimeoutExpired:watchdog=True;os.killpg(p.pid,signal.SIGKILL);p.wait()
 atomic_json(d/'execution.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog));assert p.returncode==0 and not watchdog;r=json.load(open(d/'stdout.log'));atomic_json(d/'result.json',r);v=None
 if r.get('values')is not None:v=validate(model,r);atomic_json(d/'point-validation.json',v);assert v['valid']is True
 proof=validate_cut_proofs(model,r);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True
 lines=(d/'stderr.log').read_text().splitlines();signals=[x for x in lines if any(word in x.lower()for word in ['failed','unavailable','numerical','rejected'])];record=dict(name=name,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],nodes=r['nodes'],lp_solves=r['lp_solves'],infeasibility_proof_failures=r.get('infeasibility_proof_failures'),lp_search_termination=r.get('lp_search_termination'),stderr_lines=len(lines),stderr_bytes=(d/'stderr.log').stat().st_size,signals_tail=signals[-12:],point_valid=None if v is None else True,proof_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=4))
