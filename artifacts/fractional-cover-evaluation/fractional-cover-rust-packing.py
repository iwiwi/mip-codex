import json,os,subprocess,resource,sys
from pathlib import Path
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import sha256,require_solver_origin
os.sched_setaffinity(0,set(range(33,42)));d=root/'artifacts/fractional-cover-rust-packing';d.mkdir(exist_ok=True);record=json.load(open('artifacts/fractional-cover-initial.json'));cliques=record['cliques'];n=2048
rows=[[]for _ in range(n)]
for i,group in enumerate(cliques):
 for v in group:rows[v].append(i)
lines=['NAME fractionalpacking','ROWS',' N obj']+[f' L c{i}'for i in range(len(cliques))]+['COLUMNS']
for v in range(n):
 lines.append(f' x{v} obj -1');lines.extend(f' x{v} c{i} 1'for i in rows[v])
lines+=['RHS']+[f' rhs c{i} 1'for i in range(len(cliques))]+['BOUNDS']+[f' UP b x{v} 1'for v in range(n)]+['ENDATA'];model=d/'packing.mps';model.write_text('\n'.join(lines)+'\n');binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(model),'30','1'];(d/'experiment.json').write_text(json.dumps(dict(purpose='diagnostic generated continuous LP; no native weights supplied to Rust',build_info=info,binary_sha256=sha256(binary),input_sha256=sha256(model),candidate_source_sha256=sha256('artifacts/fractional-cover-initial.json'),command=command,cpu=6,threads=1,memory_bytes=16*1024**3),indent=2)+'\n')
def restrict():
 os.sched_setaffinity(0,{6});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1')
for key in ['BINARY_GRAPH','BOX_PRIMAL','PRIMAL_REPAIR','LP_DIVING','PRIMAL_PROBE','RINS','COVER_CUTS','ROUNDING_CUTS','STRONG_BRANCHING','OBJECTIVE_ROUNDING','FEASIBILITY_PUMP']:env['MIP_'+key]='0'
with(d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,env=env,stdout=stream,stderr=err,preexec_fn=restrict,timeout=45)
(d/'exit.json').write_text(json.dumps(dict(exit_code=p.returncode))+'\n');r=json.load(open(d/'result.json'));print({k:r.get(k)for k in ['status','objective','dual_bound','seconds','lp_solves','lp_pivots']},flush=True)
