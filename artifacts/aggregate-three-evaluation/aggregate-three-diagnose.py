import json,os,sys,subprocess,resource
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256,solver_source_fingerprint
from importlib.util import spec_from_file_location,module_from_spec
cpu,part,millis=map(int,sys.argv[1:4]);os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));root=Path.cwd();work=Path('/home/ubuntu/mip-aggregate-three');binary=root/'target-aggregate-three/release/examples/separate_three';out=root/f'artifacts/aggregate-three-diagnostic-{millis}ms';out.mkdir(exist_ok=True);folders=sorted(p.parent for p in (root/'artifacts/development-root-survey-20261115').glob('*/root-lp.json'));assert len(folders)==77;fp=solver_source_fingerprint(work);spec=spec_from_file_location('three_verifier',work/'scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);records=[]
for index,d in enumerate(folders):
 if index%4!=part:continue
 meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];assert json.loads((d/'lp-audit.json').read_text())['valid']is True;directory=out/d.name;directory.mkdir(exist_ok=False);command=[str(binary),str(model),str(d/'root-lp.json'),str(millis)];atomic_json(directory/'experiment.json',dict(command=command,split='development',input_sha256=sha256(model),point_sha256=sha256(d/'root-lp.json'),binary_sha256=sha256(binary),source_fingerprint=fp,cpu=cpu,memory_bytes=16*1024**3,checker_sha256=sha256(work/'scripts/verify_cover_cuts.py')))
 with (directory/'stdout.json').open('w')as stdout,(directory/'stderr.log').open('w')as stderr:p=subprocess.run(command,stdout=stdout,stderr=stderr,timeout=30)
 assert p.returncode==0
 value=json.loads((directory/'stdout.json').read_text());assert value['library_source_fingerprint']==fp and value['source_directory']==str(work);proof=checker.validate_cut_proofs(model,value);atomic_json(directory/'proof-audit.json',proof);assert proof['valid']is True
 record=dict(name=d.name,cuts=len(value['cut_proofs']),valid=proof['valid'],seconds=value['separation_seconds'],work=value['aggregate_stats']['aggregate_work']);records.append(record);print(json.dumps(record),flush=True)
atomic_json(out/f'partition-{part}.json',dict(records=records,models=len(records),models_with_cuts=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records)));atomic_json(out/f'partition-{part}-exit.json',dict(status='complete'))
