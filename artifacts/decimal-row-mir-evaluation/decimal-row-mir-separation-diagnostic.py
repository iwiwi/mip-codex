import json,os,sys,subprocess,resource
from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256,solver_source_fingerprint
root=Path.cwd();work=Path('/home/ubuntu/mip-dual-refinement');binary=work/'target/release/examples/separate_decimal_mir';out=root/'artifacts/decimal-row-mir-separation-diagnostic';out.mkdir(exist_ok=False);os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));fp=solver_source_fingerprint(work);spec=spec_from_file_location('decimal_checker',work/'scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);folders=sorted(p.parent for p in (root/'artifacts/development-root-survey-20261115').glob('*/root-lp.json'));records=[]
# These 77 saved points were independently validated against original development MPS.
assert len(folders)==77
for d in folders:
 meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];assert json.loads((d/'lp-audit.json').read_text())['valid']is True
 for label in ['integer','decimal']:
  dest=out/d.name/label;dest.mkdir(parents=True);cmd=[str(binary),str(model),str(d/'root-lp.json'),label];atomic_json(dest/'experiment.json',dict(command=cmd,split='development',input_sha256=sha256(model),point_sha256=sha256(d/'root-lp.json'),binary_sha256=sha256(binary),source_fingerprint=fp,cpu=40,memory_bytes=16*1024**3,checker_sha256=sha256(work/'scripts/verify_cover_cuts.py')))
  with (dest/'stdout.json').open('w')as output,(dest/'stderr.log').open('w')as error:p=subprocess.run(cmd,stdout=output,stderr=error,timeout=30)
  assert p.returncode==0;value=json.loads((dest/'stdout.json').read_text());assert value['library_source_fingerprint']==fp and value['source_directory']==str(work);audit=checker.validate_cut_proofs(model,value);atomic_json(dest/'proof-audit.json',audit);assert audit['valid']is True
  scaled=sum(c['proof']['mir'].get('row_scale',1)>1 for c in value['cut_proofs']);record=dict(name=d.name,variant=label,cuts=len(value['cut_proofs']),scaled_cuts=scaled,valid=audit['valid'],seconds=value['separation_seconds']);records.append(record);print(json.dumps(record),flush=True);atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',records=len(records),cuts=sum(r['cuts']for r in records),scaled_cuts=sum(r['scaled_cuts']for r in records)))
