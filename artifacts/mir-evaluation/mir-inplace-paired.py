import json,os,resource,subprocess,time,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from benchmark import verify_snapshot
root=Path.cwd();cpu=int(sys.argv[1]);names=sys.argv[2:];os.sched_setaffinity(0,{cpu});frozen=root/'artifacts/runs/lp-inplace-final-20261116';config=json.loads((frozen/'config.json').read_text());verify_snapshot(frozen,config)
old=frozen/'snapshot/bin/mip-codex';new=root/'target/release/mip-codex';out=root/f'artifacts/mir-inplace-paired-{cpu}';out.mkdir(exist_ok=True);manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']}
atomic_json(out/'experiment.json',dict(seconds=10,cpu=cpu,threads=1,memory_bytes=16*1024**3,purpose='paired development comparison',binaries={str(p):sha256(p)for p in [old,new]}))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in names:
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256']
 for label,binary,flag in [('before',old,'0'),('after',new,'1')]:
  directory=out/name/label;directory.mkdir(parents=True,exist_ok=True);env=os.environ.copy();env.update(MIP_MIR_CUTS=flag,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1');start=time.monotonic()
  with (directory/'stderr.log').open('w')as error,(directory/'stdout.json').open('w')as output:
   run=subprocess.run([str(binary),'solve',str(model),'10'],env=env,stdout=output,stderr=error,timeout=70,preexec_fn=restrict)
  atomic_json(directory/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start))
  if run.returncode==0:
   value=json.loads((directory/'stdout.json').read_text());atomic_json(directory/'result.json',value);print(name,label,value['objective'],value['dual_bound'],value['nodes'],value['cut_stats'].get('mir_added'),flush=True)
