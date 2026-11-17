from pathlib import Path
from fractions import Fraction as F
from importlib.util import spec_from_file_location,module_from_spec
import json,random,itertools,subprocess,os,sys,signal,resource,time,copy
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/binary-equality-columns-oracle';out.mkdir();os.sched_setaffinity(0,{39});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));binary=root/'artifacts/binary-equality-columns-v1-diagnostic/bin/mip-codex';spec=spec_from_file_location('checker',root/'artifacts/binary-equality-columns-v1-diagnostic/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);rng=random.Random(2026111713);records=[];tampered=0;atomic_json(out/'experiment.json',dict(seed=2026111713,cases=200,outputs=400,cpu=39,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info'])),checker_sha256=sha256(root/'artifacts/binary-equality-columns-v1-diagnostic/scripts/verify_cover_cuts.py')))
for case in range(200):
 n=rng.randint(5,11);pairs=[(0,1)];pairs.extend((j-1,j)for j in range(2,n)if rng.random()<.35);parent=list(range(n))
 def find(j):
  while parent[j]!=j:j=parent[j]
  return j
 for j,k in pairs:parent[find(j)]=find(k)
 chosen={find(j):rng.randrange(2)for j in range(n)};plant=[chosen[find(j)]for j in range(n)];costs=[rng.randint(-9,9)for _ in range(n)];rows=[('E',f'e{k}',{i:1,j:-1},0)for k,(i,j)in enumerate(pairs)]
 for k in range(rng.randint(3,9)):
  a={j:rng.randint(-5,5)for j in rng.sample(range(n),rng.randint(2,n))};a={j:v for j,v in a.items()if v};value=sum(a.get(j,0)*x for j,x in enumerate(plant));upper=rng.randrange(2);rows.append(('L'if upper else'G',f'r{k}',a,value+(rng.randint(0,3)if upper else-rng.randint(0,3))))
 text='NAME alias\n'+('OBJSENSE\n MAX\n'if case%2 else'')+'ROWS\n N obj\n'+''.join(f' {kind} {name}\n'for kind,name,a,b in rows)+'COLUMNS\n'
 for j,c in enumerate(costs):
  text+=f' x{j} obj {c}\n'
  for kind,name,a,b in rows:
   if a.get(j):text+=f' x{j} {name} {a[j]}\n'
 text+='RHS\n'+f' rhs obj {rng.randint(-20,20)}\n'+''.join(f' rhs {name} {b}\n'for kind,name,a,b in rows)+'BOUNDS\n'+''.join(f' BV b x{j}\n'for j in range(n))+'ENDATA\n';d=out/str(case);d.mkdir();model=d/'model.mps';model.write_text(text);names,variables,constraints,objective,constant=checker.exact_model(model,True)
 def feasible(point,tol=F(0)):return all((r['lower']is None or sum(a*point[k]for k,a in r['coefficients'].items())>=r['lower']-tol)and(r['upper']is None or sum(a*point[k]for k,a in r['coefficients'].items())<=r['upper']+tol)for r in constraints)
 values=[]
 for bits in itertools.product([F(0),F(1)],repeat=n):
  p=dict(zip(names,bits))
  if feasible(p):values.append(constant+sum(a*p[k]for k,a in objective.items()))
 assert values;optimum=(max if case%2 else min)(values)
 for label in (['before','after']if case%2==0 else['after','before']):
  dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_BINARY_EQUALITY_COLUMNS='1'if label=='after'else'0',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_BINARY_GRAPH='0',MIP_PARITY_EQUALITIES='0')
  if case%3==0:
   for key in ['PRIMAL_REPAIR','BOX_PRIMAL','LP_DIVING','PRIMAL_PROBE','RINS','FEASIBILITY_PUMP','COVER_CUTS','MIR_CUTS','THREE_ROW_CUTS','CLIQUE_CUTS','AGGREGATE_CUTS']:env['MIP_'+key]='0'
  cmd=[str(binary),'solve',str(model),'2'];atomic_json(dest/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_','MKL_'))},input_sha256=sha256(model)));start=time.monotonic()
  with(dest/'result.json').open('w')as stream,(dest/'stderr.log').open('w')as error:
   proc=subprocess.Popen(cmd,stdout=stream,stderr=error,env=env,start_new_session=True)
   try:proc.wait(timeout=62)
   except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
  atomic_json(dest/'execution.json',dict(exit_code=proc.returncode,wall_seconds=time.monotonic()-start));assert proc.returncode==0;r=json.loads((dest/'result.json').read_text());proof=checker.validate_cut_proofs(model,r);assert proof['valid']is True,(case,label,proof);assert r['status']=='optimal',(case,label,r['status']);assert r['values']is not None;p={k:F(str(r['values'][k]))for k in names};assert all(-F(1,10**6)<=v<=1+F(1,10**6)and abs(v-round(v))<=F(1,10**6)for v in p.values());assert feasible(p,F(1,10**6));value=constant+sum(a*p[k]for k,a in objective.items());assert abs(value-optimum)<=F(1,10**6);assert abs(value-F(str(r['objective'])))<=F(1,10**6)
  if label=='after':
   assert r['binary_equality_proposed_columns']>0
   for kind in ['count','rows']:
    bad=copy.deepcopy(r)
    if kind=='count':bad['binary_equality_proposed_columns']+=1
    else:bad['binary_equality_proof']['rows']=[]
    check=checker.validate_cut_proofs(model,bad);assert check['valid']is False;(dest/('tampered-'+kind+'.json')).write_text(json.dumps(check,indent=2)+'\n');tampered+=1
  rec=dict(case=case,label=label,maximize=bool(case%2),status=r['status'],objective=r['objective'],optimum=str(optimum),point_valid=True,proof_valid=True,aliases=r['binary_equality_proposed_columns'],nodes=r['nodes'],valid=True);records.append(rec);atomic_json(dest/'validation.json',rec);atomic_json(out/'progress.json',dict(outputs=len(records),valid=len(records),points=len(records),alias_outputs=sum(x['aliases']>0 for x in records),tampered_rejected=tampered))
atomic_json(out/'records.json',records);summary=json.loads((out/'progress.json').read_text());atomic_json(out/'summary.json',summary);atomic_json(out/'exit.json',dict(status='complete'));print(summary)
