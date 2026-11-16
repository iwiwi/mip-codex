import json,os,resource,subprocess,sys,time
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from verify_cover_cuts import validate_cut_proofs
root=Path.cwd();part=int(sys.argv[1]);cpu=int(sys.argv[2]);os.sched_setaffinity(0,set(range(33,42)));out=root/'artifacts/clique-cut-rust-diagnostic';out.mkdir(exist_ok=True);binary=root/'target/release/examples/separate_clique';folders=sorted(d.parent for d in (root/'artifacts/development-root-survey-20261115').glob('*/root-lp.json'));assert len(folders)==77
records=[]
def restrict():os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for case,d in enumerate(folders):
 if case%4!=part:continue
 meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];dest=out/d.name;dest.mkdir(exist_ok=True);command=[str(binary),str(model),str(d/'root-lp.json')]
 atomic_json(dest/'experiment.json',dict(command=command,cpu=cpu,threads=1,memory_bytes=16*1024**3,binary_sha256=sha256(binary),source_sha256=sha256(root/'src/cuts/clique.rs'),model_sha256=sha256(model),point_sha256=sha256(d/'root-lp.json'),split='development',purpose='Rust separator only on saved own LP point'))
 with(dest/'result.json').open('w')as stream,(dest/'stderr.log').open('w')as err:r=subprocess.run(command,stdout=stream,stderr=err,preexec_fn=restrict,timeout=60)
 atomic_json(dest/'exit.json',dict(exit_code=r.returncode));assert r.returncode==0
 result=json.loads((dest/'result.json').read_text());proof=validate_cut_proofs(model,result);assert proof['valid'];atomic_json(dest/'proof.json',proof);record=dict(name=d.name,cuts=len(result['cut_proofs']),proof=proof,stats=result['clique_stats'],seconds=result['separation_seconds']);records.append(record);print(json.dumps(dict(name=d.name,cuts=record['cuts'],seconds=record['seconds'])),flush=True)
atomic_json(out/f'partition-{part}.json',dict(records=records,models=len(records),models_with_cuts=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records)))
