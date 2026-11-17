from pathlib import Path
import sys,json,os,subprocess,shutil,resource,signal
sys.path.insert(0,'scripts')
from benchmark_common import require_solver_origin,atomic_json,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-objective-repair');binary=repo/'target/release/examples/cover_structure';info=require_solver_origin(repo,binary);out=root/'artifacts/cover-structure-scan';out.mkdir();(out/'bin').mkdir();shutil.copy2(binary,out/'bin/helper');shutil.copy2(repo/'examples/cover_structure.rs',out/'helper.rs');atomic_json(out/'experiment.json',dict(cpu=39,memory_bytes=16*1024**3,build_info=info,binary_sha256=sha256(binary),purpose='development-only structural diagnostics; no optimization, no holdout model reads',source_reference='artifacts/component-work-share-default-on-diagnostic'));os.sched_setaffinity(0,{39});records=[]
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for r in json.load(open('data/manifest.json'))['instances']:
 if r['split']!='development':continue
 p=root/r['path'];assert sha256(p)==r['sha256'];d=out/r['name'];d.mkdir()
 with(d/'stdout.json').open('w')as stream,(d/'stderr.log').open('w')as err:
  proc=subprocess.Popen([str(out/'bin/helper'),str(p)],stdout=stream,stderr=err,preexec_fn=restrict,start_new_session=True)
  try:proc.wait(timeout=60)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
 v=dict(name=r['name'],input_sha256=r['sha256'],exit_code=proc.returncode)
 if proc.returncode==0:v.update(json.load(open(d/'stdout.json')))
 records.append(v);atomic_json(d/'record.json',v);atomic_json(out/'records.json',records);print(json.dumps(v),flush=True)
atomic_json(out/'exit.json',dict(status='complete',records=len(records)))
