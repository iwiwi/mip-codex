from pathlib import Path
import json,os,sys,resource
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute'
from validate_solution import validate
os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));source=Path('artifacts/reusable-repair-mip-oracle');out=Path('artifacts/reusable-repair-mip-point-audit');out.mkdir();records=[]
atomic_json(out/'experiment.json',dict(source=str(source),cpu=40,memory_bytes=16*1024**3,purpose='all synthetic MIP output points against original MPS with absolute GMP checker',checker_sha256=sha256(Path('/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute'))))
try:
 for p in sorted(source.glob('*/*/result.json')):
  r=json.loads(p.read_text());assert r['values']is not None;model=p.parent.parent/'model.mps';v=validate(model,r);d=out/p.parent.parent.name/p.parent.name;d.mkdir(parents=True);atomic_json(d/'validation.json',v);assert v['valid']is True,(str(p),v);records.append(dict(case=p.parent.parent.name,variant=p.parent.name,input_sha256=sha256(model),result_sha256=sha256(p),valid=v['valid']));atomic_json(out/'records.json',records)
  if len(records)%100==0:print(len(records),flush=True)
 assert len(records)==800;atomic_json(out/'summary.json',dict(points=800,strict_valid=800,invalid=0));atomic_json(out/'exit.json',dict(status='complete'))
except BaseException as error:atomic_json(out/'exit.json',dict(status='failed',completed=len(records),error=str(error)));raise
