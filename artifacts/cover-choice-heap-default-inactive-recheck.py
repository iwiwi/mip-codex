from pathlib import Path
import os,sys,json,subprocess,time,resource
root=Path.cwd()
os.sched_setaffinity(0,{46})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
binary=root/'artifacts/cover-choice-heap-default-frozen/bin/mip-codex'
origin=json.load(open('artifacts/cover-choice-heap-default-origin.json'))
assert sha256(binary)==origin['binary_sha256']
manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']if r['split']=='development'}
names=['neos-911970.mps.gz']
out=root/'artifacts/cover-choice-heap-default-inactive-recheck'
out.mkdir()
atomic_json(out/'experiment.json',dict(cpu=46,threads=1,memory_bytes=16*1024**3,origin=origin,seconds_each=10,trials=3,purpose='retain final188 losses; alternating same CPU inactive-path repeat, no source changes'))
records=[]
for name in names:
    item=manifest[name]
    model=root/item['path']
    assert sha256(model)==item['sha256']
    for trial in range(3):
        for flag in ([0,1]if trial%2==0 else[1,0]):
            d=out/name/str(trial)/str(flag)
            d.mkdir(parents=True)
            env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
            if flag==0:
                env['MIP_GRAPH_COVER_HEAP']='0'
            cmd=[str(binary),'solve',str(model),'10']
            atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')}))
            start=time.monotonic()
            run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=70)
            (d/'stdout.log').write_text(run.stdout)
            (d/'stderr.log').write_text(run.stderr)
            atomic_json(d/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start))
            assert run.returncode==0,run.stderr
            r=json.loads(run.stdout)
            atomic_json(d/'result.json',r)
            assert r['binary_graph']is None
            v=None
            if r.get('values')is not None:
                v=validate(model,r)
                atomic_json(d/'validation.json',v)
                assert v['valid']
            p=validate_cut_proofs(model,r)
            atomic_json(d/'proof.json',p)
            assert p['valid']
            record=dict(name=name,trial=trial,flag=flag,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],seconds=r['seconds'],point_valid=None if v is None else v['valid'],proof_valid=True)
            records.append(record)
            atomic_json(out/'records.json',records)
            print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=len(records),gmp_valid=sum(r['point_valid']is True for r in records),proof_valid=len(records)))
