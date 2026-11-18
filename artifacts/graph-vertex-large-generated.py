from pathlib import Path
import os,sys,json,subprocess,random,resource,itertools
root=Path.cwd()
os.sched_setaffinity(0,{49})
resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
from verify_cover_cuts import validate_cut_proofs
binary=root/'artifacts/graph-vertex-frozen/bin/mip-codex'
origin=json.load(open('artifacts/graph-vertex-origin.json'))
assert sha256(binary)==origin['binary_sha256']
out=root/'artifacts/graph-vertex-large-generated'
out.mkdir()
atomic_json(out/'experiment.json',dict(cpu=49,threads=1,memory_bytes=16*1024**3,origin=origin,seconds_each=5,random_seed=202611180702,purpose='large disconnected random small cores; validator sums exhaustive core optima; no targets passed to Rust'))
rng=random.Random(202611180702)
records=[]
for case in range(6):
    n=4200+100*case
    edges=[]
    offset=0
    alpha=0
    while offset<n:
        if offset<500:
            size=5
            local=[(k,(k+1)%5)for k in range(5)]
        else:
            size=min(n-offset,rng.randrange(4,9))
            local=[(a,b)for a in range(size)for b in range(a+1,size)if rng.random()<.35]
        optimum=max(sum(x)for x in itertools.product([0,1],repeat=size)if all(x[a]+x[b]<=1 for a,b in local))
        alpha+=optimum
        edges.extend((offset+a,offset+b)for a,b in local)
        offset+=size
    terms=[[]for _ in range(n)]
    for k,(a,b)in enumerate(edges):
        terms[a].append(k)
        terms[b].append(k)
    maximize=bool(case%2)
    direction=-1 if case%3==1 else 1
    model=out/f'{case:02d}.mps'
    lines=['NAME HEAPTEST','OBJSENSE',' MAX'if maximize else' MIN','ROWS',' N obj']
    lines.extend(f' {"G"if direction<0 else"L"} r{k}'for k in range(len(edges)))
    lines.append('COLUMNS')
    for v in range(n):
        lines.append(f' x{v} obj {3 if maximize else -3}')
        lines.extend(f' x{v} r{k} {direction}'for k in terms[v])
    lines.extend(['RHS',' rhs obj -7'])
    lines.extend(f' rhs r{k} {direction}'for k in range(len(edges)))
    lines.append('BOUNDS')
    lines.extend(f' BV b x{v}'for v in range(n))
    lines.append('ENDATA')
    model.write_text('\n'.join(lines)+'\n')
    for flag in [0,1]:
        d=out/f'{case:02d}-{flag}'
        d.mkdir()
        env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')}
        env['MIP_GRAPH_VERTEX_PORTFOLIO']=str(flag)
        cmd=[str(binary),'solve',str(model),'5']
        atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')},exhaustive_component_alpha=alpha))
        run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=65)
        (d/'stdout.log').write_text(run.stdout)
        (d/'stderr.log').write_text(run.stderr)
        assert run.returncode==0,run.stderr
        r=json.loads(run.stdout)
        atomic_json(d/'result.json',r)
        g=r['binary_graph']
        assert g is not None
        assert g['best']<=alpha<=g['upper'],(case,flag,alpha,g['best'],g['upper'])
        if r['status']=='optimal':
            assert g['best']==alpha
        point=validate(model,r)
        atomic_json(d/'point-validation.json',point)
        assert point['valid']
        proof=validate_cut_proofs(model,r)
        atomic_json(d/'proof-validation.json',proof)
        assert proof['valid']
        used=bool((g.get('cover_local')or{}).get('heap_used'))
        assert used
        long_used=bool((g.get('cover_local')or{}).get('long_rounds_used'))
        assert long_used
        vertex_used=bool((g.get('vertex_local')or{}).get('used'));assert vertex_used==(flag==1)
        assert (g.get('vertex_local')or{}).get('validation_failures',0)==0
        record=dict(case=case,flag=flag,n=n,edges=len(edges),alpha=alpha,best=g['best'],upper=g['upper'],status=r['status'],seconds=r['seconds'],heap_used=used,long_rounds_used=long_used,vertex_used=vertex_used,gmp_valid=True,proof_valid=True)
        records.append(record)
        atomic_json(out/'records.json',records)
        print(json.dumps(record),flush=True)
assert any(r['heap_used']for r in records)
atomic_json(out/'summary.json',dict(outputs=len(records),gmp_valid=len(records),proof_valid=len(records),actual_long_round_outputs=sum(r['long_rounds_used']for r in records),actual_vertex_outputs=sum(r['vertex_used']for r in records),actual_heap_outputs=sum(r['heap_used']for r in records),exhaustive_component_bound_valid=len(records)))
