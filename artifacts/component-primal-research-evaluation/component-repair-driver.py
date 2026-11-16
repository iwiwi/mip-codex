from pathlib import Path
import sys,os,subprocess,shutil,json,time,resource,signal
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/component-repair-cygnet-30';out.mkdir();source=root/'artifacts/component-primal-cygnet-120/components/component-00000/model.json';binary=out/'repair-component';shutil.copy2(repo/'target/release/examples/repair_component',binary);shutil.copy2(repo/'examples/repair_component.rs',out/'repair_component.rs');os.sched_setaffinity(0,{1});cmd=[str(binary),str(source),'30',str(out/'result.json')];env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');atomic_json(out/'experiment.json',dict(command=cmd,cpu=1,seconds=30,watchdog_seconds=90,memory_bytes=16*1024**3,input_sha256=sha256(source),binary_sha256=sha256(binary),source_sha256=sha256(out/'repair_component.rs'),purpose='LP-free existing Rust primal repair diagnostic; no global proof or main change'))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
start=time.monotonic()
with(out/'stdout.log').open('w')as f,(out/'stderr.log').open('w')as e:
 p=subprocess.Popen(cmd,env=env,stdout=f,stderr=e,preexec_fn=restrict,start_new_session=True)
 try:p.wait(timeout=90);watchdog=False
 except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
 except BaseException:
  if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
  raise
atomic_json(out/'exit.json',dict(exit_code=p.returncode,wall_seconds=time.monotonic()-start,watchdog=watchdog));print(json.load(open(out/'exit.json')))
