import os,json,subprocess,resource,time,signal,sys
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();source=root/'artifacts/component-primal-cygnet-120/components/component-00000/model.json';v=json.load(open(source));assert not v['maximize'] and v['offset']==0;assert all(r['exact_integral']and r['exact_coefficients']for r in v['rows']);assert all(x['objective']==1 and x['lower']==0 and x['upper']==1 and x['integer']for x in v['variables']);out=root/'artifacts/component-phase-one-cygnet-60';out.mkdir();lines=['NAME component','OBJSENSE',' MIN','ROWS',' N  obj'];adj=[[]for _ in v['variables']]
for i,r in enumerate(v['rows']):
 assert (r['lower'],r['upper'])in [(1,None),(None,2)];lines.append(f' {"G"if r["lower"]is not None else"L"}  r{i:07}')
 for j,a in r['coefficients']:assert a==1;adj[j].append(i)
lines.append('COLUMNS')
for j,a in enumerate(adj):
 lines.append(f'    x{j:07}  {"obj":<8}  {1:>12}')
 for i in a:lines.append(f'    x{j:07}  r{i:07}  {1:>12}')
lines.append('RHS')
for i,r in enumerate(v['rows']):lines.append(f'    {"rhs":<8}  r{i:07}  {int(r["lower"]or r["upper"]):>12}')
lines+=['BOUNDS']+[f' BV {"bounds":<8}  x{j:07}'for j in range(len(adj))]+['ENDATA'];model=out/'component.mps';model.write_text('\n'.join(lines)+'\n');binary=root/'artifacts/phase-one-keep-pricing-final-diagnostic/bin/mip-codex';info=json.loads(subprocess.check_output([str(binary),'build-info']));assert info['source_fingerprint']=='b0767d78d5087e0e';os.sched_setaffinity(0,{42});atomic_json(out/'experiment.json',dict(source_model_sha256=sha256(source),input_sha256=sha256(model),binary_sha256=sha256(binary),build_info=info,seconds=60,cpu=42,memory_bytes=16*1024**3,threads=1,purpose='isolated Phase I pricing on exact integer disconnected component; no tuning holdout'))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for flag in ['0','1']:
 d=out/flag;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_PHASE1_KEEP_PRICING=flag,MIP_ROOT_LP_DUMP=str(d/'root-lp.json'))
 for f in ['PARITY_EQUALITIES','THREE_ROW_CUTS','MIR_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS','COVER_CUTS','ROUNDING_CUTS','BINARY_GRAPH','OBJECTIVE_ROUNDING','BOX_PRIMAL','PRIMAL_REPAIR','PRIMAL_PROBE','RINS','STRONG_BRANCHING','LP_DIVING','FEASIBILITY_PUMP','PROPAGATION']:env['MIP_'+f]='0'
 atomic_json(d/'environment.json',{k:x for k,x in env.items()if k.startswith('MIP_')or k in ['OMP_NUM_THREADS','OPENBLAS_NUM_THREADS','MKL_NUM_THREADS']});start=time.monotonic()
 with(d/'result.json').open('w')as f,(d/'stderr.log').open('w')as e:
  p=subprocess.Popen([str(binary),'solve',str(model),'60','1'],env=env,stdout=f,stderr=e,start_new_session=True,preexec_fn=restrict)
  try:p.wait(timeout=120);watchdog=False
  except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGKILL);p.wait();watchdog=True
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 atomic_json(d/'exit.json',dict(exit_code=p.returncode,watchdog=watchdog,wall_seconds=time.monotonic()-start))
 if p.returncode==0:
  r=json.load(open(d/'result.json'));print(flag,{k:r.get(k)for k in ['status','dual_bound','lp_pivots','seconds']},flush=True)
atomic_json(out/'exit.json',dict(status='complete'))
