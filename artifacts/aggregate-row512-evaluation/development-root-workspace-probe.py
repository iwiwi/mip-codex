"""Frozen root-only workspace diagnostic; no solver code or proof changes."""
import json,os,resource,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256
root=Path.cwd();part=int(sys.argv[1]);cpu=int(sys.argv[2]);os.sched_setaffinity(0,set(range(37,42)))
snapshot=root/'artifacts/runs/aggregate-cuts-final-20261115';c=json.loads((snapshot/'config.json').read_text());verify_snapshot(snapshot,c);binary=snapshot/'snapshot/bin/mip-codex';records=[]
folders=sorted(p.parent for p in Path('artifacts/development-root-survey-20261115').glob('*/summary.json')if json.loads(p.read_text()).get('status')=='resource_limit');assert len(folders)==11
for i,source in enumerate(folders):
 if i%2!=part:continue
 meta=json.loads((source/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',**meta['environment']);env['MIP_LP_WORKSPACE_MIB']='2048'
 d=root/'artifacts/development-root-workspace2048'/source.name;d.mkdir(parents=True,exist_ok=True);command=[str(binary),'solve',str(model),'10','1'];point=d/'root-lp.json';atomic_json(d/'experiment.json',dict(meta,command=command,purpose='workspace limit diagnostic only, not final benchmark',cpu=cpu,environment={k:v for k,v in env.items()if k.startswith('MIP_')},baseline_status='resource_limit',baseline_workspace_mib=512,workspace_mib=2048))
 def restrict():
  os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
 start=time.monotonic()
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:
  try:p=subprocess.run(command,env=dict(env,MIP_ROOT_LP_DUMP=str(point)),preexec_fn=restrict,stdout=out,stderr=err,timeout=60);execution=dict(exit_code=p.returncode)
  except subprocess.TimeoutExpired:execution=dict(external_watchdog=True)
 execution['wall_seconds']=time.monotonic()-start;atomic_json(d/'exit.json',execution)
 try:r=json.loads((d/'result.json').read_text());record={k:r.get(k)for k in ['status','objective','dual_bound','nodes','seconds','lp_solves','lp_pivots','lp_workspace_bytes']};record.update(name=source.name,point_dump=point.is_file(),execution=execution)
 except Exception as e:record=dict(name=source.name,error=str(e),execution=execution)
 atomic_json(d/'summary.json',record);records.append(record);print(json.dumps(record),flush=True)
atomic_json(root/f'artifacts/development-root-workspace2048-partition-{part}.json',records)
