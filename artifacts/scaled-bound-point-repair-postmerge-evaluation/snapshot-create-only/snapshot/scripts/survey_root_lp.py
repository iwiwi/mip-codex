#!/usr/bin/env python3
"""Frozen Rust root-only development survey; diagnostic, never a final benchmark."""
import argparse,json,os,resource,subprocess,time
from pathlib import Path
from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256

parser=argparse.ArgumentParser();parser.add_argument('directory',type=Path);parser.add_argument('--snapshot',type=Path,required=True);parser.add_argument('--partition',type=int,required=True);parser.add_argument('--partitions',type=int,default=4);parser.add_argument('--cpu',type=int,required=True);parser.add_argument('--seconds',type=int,default=10);args=parser.parse_args()
assert 0<=args.partition<args.partitions and args.cpu in range(8) and args.seconds>0
root=Path(__file__).resolve().parents[1];snapshot=args.snapshot.resolve();config=json.loads((snapshot/'config.json').read_text());verify_snapshot(snapshot,config)
binary=snapshot/'snapshot/bin/mip-codex';info=json.loads(subprocess.check_output([str(binary),'build-info'],text=True));assert info==config['solver_build_info']
instances=[r for r in config['instances']if r['split']=='development'];assert len(instances)==188
selected=[r for i,r in enumerate(instances)if i%args.partitions==args.partition]
args.directory.mkdir(parents=True,exist_ok=True);os.sched_setaffinity(0,set(range(33,42)))
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_LP_ENGINE='revised',MIP_ABSOLUTE_CHECKER=str(snapshot/'snapshot/bin/solchecker-absolute'))
for flag in ['MIP_MIR_CUTS','MIP_CLIQUE_CUTS','MIP_AGGREGATE_CUTS','MIP_BINARY_GRAPH','MIP_COVER_CUTS','MIP_ROUNDING_CUTS','MIP_LP_DIVING','MIP_PRIMAL_REPAIR','MIP_PRIMAL_PROBE','MIP_RINS','MIP_FEASIBILITY_PUMP','MIP_STRONG_BRANCHING','MIP_OBJECTIVE_ROUNDING','MIP_PROPAGATION','MIP_BOX_PRIMAL']:env[flag]='0'
def restrict():
 os.sched_setaffinity(0,{args.cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
records=[]
for instance in selected:
 assert instance['split']=='development';model=Path(instance['absolute_path']);assert sha256(model)==instance['sha256']
 d=args.directory/instance['name'].removesuffix('.mps.gz');d.mkdir(exist_ok=True)
 if(d/'summary.json').is_file():records.append(json.loads((d/'summary.json').read_text()));continue
 # Refuse to overwrite an interrupted attempt or its create-new dump.
 assert not(d/'experiment.json').exists(),f'interrupted attempt requires a new destination: {d}'
 command=[str(binary),'solve',str(model),str(args.seconds),'1'];point=(d/'root-lp.json').resolve()
 atomic_json(d/'experiment.json',dict(purpose='development root LP diagnostic, not final benchmark',command=command,input_sha256=instance['sha256'],split='development',build_info=info,binary_sha256=sha256(binary),frozen_config_sha256=sha256(snapshot/'config.json'),cpu=args.cpu,threads=1,memory_bytes=16*1024**3,memory_policy='per-process RLIMIT_AS; diagnostic only',environment={k:v for k,v in env.items()if k.startswith('MIP_')},partition=args.partition))
 start=time.monotonic()
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:
  try:p=subprocess.run(command,env=dict(env,MIP_ROOT_LP_DUMP=str(point)),preexec_fn=restrict,stdout=out,stderr=err,timeout=args.seconds+50);execution=dict(exit_code=p.returncode,external_watchdog=False)
  except subprocess.TimeoutExpired:execution=dict(external_watchdog=True,timeout_seconds=args.seconds+50)
 execution['wall_seconds']=time.monotonic()-start;atomic_json(d/'exit.json',execution)
 record=dict(name=instance['name'],input_sha256=instance['sha256'],split='development',execution=execution,point_dump=point.is_file())
 try:
  result=json.loads((d/'result.json').read_text());record.update({k:result.get(k)for k in ['status','objective','dual_bound','nodes','seconds','lp_pivots','lp_refactors']})
  if point.is_file():
   data=json.loads(point.read_text());assert len(data['names'])==len(data['point'])and len(set(data['names']))==len(data['names']);record['lp_objective']=data['objective'];record['variables']=len(data['point'])
  if result.get('values')is not None:
   with(d/'validation.log').open('w')as out:q=subprocess.run([str(root/'.venv/bin/python'),str(root/'scripts/benchmark_validate.py'),str(model),str(d/'result.json'),str(snapshot/'snapshot/bin/solchecker'),str(d)],env=env,stdout=out,stderr=subprocess.STDOUT,timeout=600)
   record['validation_exit']=q.returncode;record['strict_validation']=json.loads((d/'strict-validation.json').read_text())
 except Exception as error:record['diagnostic_error']=f'{type(error).__name__}: {error}'
 atomic_json(d/'summary.json',record);records.append(record);print(json.dumps(record),flush=True)
atomic_json(args.directory/f'partition-{args.partition}-summary.json',dict(purpose='collection completion, not every solver case passed',instances=len(records),points=sum(r['point_dump']for r in records),records=records))
