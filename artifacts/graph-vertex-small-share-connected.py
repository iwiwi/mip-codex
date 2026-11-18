from pathlib import Path
import os,sys,json,random,subprocess,resource,time
os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2)
os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute')
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate;from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/graph-vertex-small-share-connected';out.mkdir();binary=root/'artifacts/graph-vertex-small-share-frozen/bin/mip-codex';origin=json.load(open('artifacts/graph-vertex-small-share-origin.json'));assert sha256(binary)==origin['binary_sha256']
atomic_json(out/'experiment.json',dict(origin=origin,cpu=45,seconds=20,threads=1,memory_bytes=16*1024**3,seed=202611181300,models=12,n=5000,construction='cycle plus uniform random simple edges until requested mean degree; connected by construction',degrees=[6,20,100],targets=None,purpose='connected generic packing quality controls; no known optimum used or claimed; all point and certificate outputs independently checked'))
records=[]
for case in range(12):
 n=5000;degree=[6,20,100][case%3];rng=random.Random(202611181300+case);edges={(a,a+1)for a in range(n-1)};edges.add((0,n-1))
 while len(edges)<n*degree//2:
  a=rng.randrange(n);b=rng.randrange(n)
  if a!=b:edges.add((min(a,b),max(a,b)))
 edges=sorted(edges);incident=[[]for _ in range(n)]
 for k,(a,b)in enumerate(edges):incident[a].append(k);incident[b].append(k)
 model=out/f'{case:02d}.mps';lines=['NAME CONNECT','ROWS',' N obj']+[f' L r{k}'for k in range(len(edges))]+['COLUMNS']
 for a in range(n):lines.append(f' x{a} obj -1');lines.extend(f' x{a} r{k} 1'for k in incident[a])
 lines+=['RHS']+[f' rhs r{k} 1'for k in range(len(edges))]+['BOUNDS']+[f' BV b x{a}'for a in range(n)]+['ENDATA'];model.write_text('\n'.join(lines)+'\n')
 for flag in ([0,1]if case%2==0 else[1,0]):
  d=out/f'{case:02d}-{flag}';d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env['MIP_GRAPH_VERTEX_PORTFOLIO']=str(flag);cmd=[str(binary),'solve',str(model),'20'];atomic_json(d/'job.json',dict(command=cmd,flag=flag,input_sha256=sha256(model),cpu=45));start=time.monotonic();run=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=80);(d/'stdout.log').write_text(run.stdout);(d/'stderr.log').write_text(run.stderr);assert run.returncode==0,run.stderr;r=json.loads(run.stdout);atomic_json(d/'result.json',r)
  point=validate(model,r);proof=validate_cut_proofs(model,r);atomic_json(d/'point-validation.json',point);atomic_json(d/'proof-validation.json',proof);assert point['valid']is True and proof['valid']is True
  rec=dict(case=case,degree=degree,flag=flag,objective=r['objective'],dual_bound=r['dual_bound'],status=r['status'],seconds=r['seconds'],wall_seconds=time.monotonic()-start,vertex=(r.get('binary_graph')or{}).get('vertex_local'),point_valid=True,proof_valid=True);records.append(rec);atomic_json(out/'records.json',records);print(json.dumps(rec),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=len(records),gmp_valid=len(records),proof_valid=len(records)))
