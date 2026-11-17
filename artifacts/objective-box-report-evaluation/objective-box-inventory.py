from pathlib import Path
import json,subprocess,hashlib,os,resource,sys
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/objective-box-inventory';out.mkdir();os.sched_setaffinity(0,{41});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=Path('/home/ubuntu/mip-dual-restoration/target/release/examples/objective_box_inventory');source=Path('/home/ubuntu/mip-dual-restoration/examples/objective_box_inventory.rs');(out/'helper.rs').write_bytes(source.read_bytes());atomic_json(out/'experiment.json',dict(purpose='read-only development objective box inventory; not adoption',cpu=41,binary_sha256=sha256(binary),helper_source_sha256=sha256(source),library_source_reference='main PR105 FP04a5bfb314092d16'));records=[]
manifest=[r for r in json.loads((root/'data/manifest.json').read_text())['instances']if r['split']=='development'];assert len(manifest)==188
try:
 for r in manifest:
  p=root/r['path'];assert sha256(p)==r['sha256'];d=out/r['name'];d.mkdir()
  with(d/'stdout.json').open('w')as stream,(d/'stderr.log').open('w')as error:proc=subprocess.run([str(binary),str(p)],stdout=stream,stderr=error,timeout=60)
  result=dict(name=r['name'],input_sha256=r['sha256'],exit_code=proc.returncode)
  if proc.returncode==0:
   v=json.loads((d/'stdout.json').read_text());result.update(v);comparison=root/'artifacts/safe-gmi-work-cap-development-screen/cases'/r['name']/'after/result.json';old=json.loads(comparison.read_text());obj=old.get('objective');bound=v['box_normalized_lower'];sign=-1 if old.get('maximize',False)else 1
   # MIPLIB development metadata sense is read from the original MPS, not a guessed result field.
   from verify_binary_graph import maximize_sense
   sign=-1 if maximize_sense(p)else 1
   result.update(primal=obj,reported_dual=old.get('dual_bound'),box_attainment_candidate=obj is not None and bound is not None and abs(sign*obj-bound)<=1e-10*(1+abs(obj)))
  records.append(result);atomic_json(d/'record.json',result);atomic_json(out/'records.json',records)
 summary=dict(cases=len(records),success=sum(r['exit_code']==0 for r in records),finite_boxes=sum(r.get('box_normalized_lower')is not None for r in records),constant_objectives=sum(r.get('constant_objective',False)for r in records),attainment_candidates=[r['name']for r in records if r.get('box_attainment_candidate')]);atomic_json(out/'summary.json',summary);atomic_json(out/'exit.json',dict(status='complete'));print(summary)
except BaseException as error:atomic_json(out/'exit.json',dict(status='failed',completed=len(records),error=str(error)));raise
