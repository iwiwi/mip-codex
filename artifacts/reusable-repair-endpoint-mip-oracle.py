from pathlib import Path
from fractions import Fraction as F
import os,sys,json,subprocess,random,itertools,copy,struct,time,resource
root=Path.cwd();frozen=root/'artifacts/reusable-repair-frozen';sys.path.insert(0,str(frozen/'scripts'));from verify_cover_cuts import exact_model,validate_cut_proofs
sys.path.insert(0,str(root/'scripts'));from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{38});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));out=root/'artifacts/reusable-repair-endpoint-mip-oracle';out.mkdir();binary=frozen/'bin/mip-codex';rng=random.Random(2026111726);records=[];tampered=0
atomic_json(out/'experiment.json',dict(seed=2026111726,cpu=41,models=100,senses=['MIN','MAX'],flags=[0,1],seconds=2,extra_zero_cost_empty_continuous_columns=1500,correctness_only_workspace_mib=dict(before=1,after=1),purpose='force original guard and no initial incumbent; feasibility subset primal off/on; exact correctness only',memory_bytes=16*1024**3,binary_sha256=sha256(binary),build_info=json.loads(subprocess.check_output([str(binary),'build-info']))))
for case in range(100):
 costs=[rng.choice([i for i in range(-9,10)if i])for _ in range(3)]+[0,0];plant=[rng.randrange(2)for _ in costs];rows=[];offset=rng.randint(-20,20)
 for i in range(5):
  a=[rng.randint(-3,3)for _ in range(3)]+[0,0]
  if i>=3:a[i]=rng.choice([-3,-2,-1,1,2,3])
  if not any(a):a[0]=1
  sense=rng.choice(['L','G']);rhs=sum(c*x for c,x in zip(a,plant))+(1 if sense=='L'else-1)*rng.randint(0,2);rows.append((a,sense,rhs))
 for j in [3,4]:
  a=[rng.randint(-3,3)for _ in range(3)]+[0,0];a[j]=2*rows[j][0][j];sense=rows[j][1];rhs=sum(c*x for c,x in zip(a,plant))+(1 if sense=='L'else-1)*rng.randint(0,2);rows.append((a,sense,rhs))
 a=[rng.randint(-3,3)for _ in range(3)]+[0,0]
 for j in [3,4]:
  old_a,old_sense,_=rows[j];upper=(old_a[j]<0)if old_sense=='L'else(old_a[j]>0);a[j]=-2 if upper else 2
 rows.append((a,'L',sum(c*x for c,x in zip(a,plant))+rng.randint(0,2)))
 points=[x for x in itertools.product(range(2),repeat=5)if all((sum(c*v for c,v in zip(a,x))<=b if sense=='L'else sum(c*v for c,v in zip(a,x))>=b)for a,sense,b in rows)];assert points
 rows += [([1,0,0,0,0],'L',1000+i)for i in range(800)]
 for maximize in [False,True]:
  d=out/f'{case}-{int(maximize)}';d.mkdir();model=d/'model.mps';text='NAME dom\nOBJSENSE\n '+('MAX'if maximize else'MIN')+'\nROWS\n N obj\n'+''.join(f' {s} r{i}\n'for i,(_,s,_)in enumerate(rows))+'COLUMNS\n'
  for j,c in enumerate(costs):text+=f' x{j} obj {c}\n'+''.join(f' x{j} r{i} {a[j]}\n'for i,(a,_,_)in enumerate(rows)if a[j])
  text+=''.join(f' pad{j} obj 0\n'for j in range(1500))
  text+='RHS\n'+f' rhs obj {-offset}\n'+''.join(f' rhs r{i} {b}\n'for i,(_,_,b)in enumerate(rows))+'BOUNDS\n'+''.join(f' LI b x{j} 0\n UI b x{j} 1\n'for j in range(5))+''.join(f' LO b pad{j} 0\n UP b pad{j} 2\n'for j in range(1500))+'ENDATA\n';model.write_text(text);names,variables,parsed_rows,obj,constant=exact_model(model,True);assert constant==offset and all(obj.get(f'x{j}',F(0))==c for j,c in enumerate(costs));opt=(max if maximize else min)(offset+sum(c*v for c,v in zip(costs,x))for x in points)
  for label in (['before','after']if case%2==0 else['after','before']):
   dest=d/label;dest.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_ADAPTIVE_REPAIR_REUSE='1',MIP_ADAPTIVE_RESOURCE_REPAIR='0'if label=='before'else'1',MIP_ROW_GENERATION='1',MIP_ROW_GENERATION_REPORT='1',MIP_LP_WORKSPACE_MIB='1',MIP_BOX_PRIMAL='0',MIP_PRIMAL_REPAIR='0',MIP_BINARY_GRAPH='0',OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1');cmd=[str(binary),'solve',str(model),'2'];atomic_json(dest/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_'))},input_sha256=sha256(model)));start=time.monotonic();run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=62);(dest/'result.json').write_text(run.stdout);(dest/'stderr.log').write_text(run.stderr);atomic_json(dest/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start));assert run.returncode==0;r=json.loads(run.stdout);check=validate_cut_proofs(model,r);atomic_json(dest/'proof-validation.json',check);assert check['valid']is True,(case,maximize,label,check);assert r['status']=='optimal',(case,maximize,label,r['status']);assert abs(F(str(r['objective']))-opt)<=F(1,10**8)*(1+abs(opt));x={name:F(str(r['values'][name]))for name in names};assert all((v['lower']is None or z>=v['lower']-F(1,1000000))and(v['upper']is None or z<=v['upper']+F(1,1000000))and(not v['integer']or abs(z-round(z))<=F(1,1000000))for name,z in x.items()for v in [variables[name]]);assert all((row['lower']is None or sum(a*x[k]for k,a in row['coefficients'].items())>=row['lower']-F(1,1000000))and(row['upper']is None or sum(a*x[k]for k,a in row['coefficients'].items())<=row['upper']+F(1,1000000))for row in parsed_rows)
   if r['dual_bound']is not None:assert (-1 if maximize else 1)*F(str(r['dual_bound']))<=(-1 if maximize else 1)*opt
   if r['row_generation'].get('bound_proof')is not None:
    for kind in ['duplicate','range','empty','extra','bound','nonfinite','zero','wrongside']:
     bad=copy.deepcopy(r);p=bad['row_generation']['bound_proof']
     if kind=='duplicate':p['multipliers'].append(p['multipliers'][-1])
     elif kind=='range':p['multipliers'][0][0]=len(parsed_rows)
     elif kind=='empty':p['multipliers']=[]
     elif kind=='extra':p['extra']=True
     elif kind=='bound':p['normalized_bound_bits']=struct.pack('>d',float((-1 if maximize else 1)*opt+1)).hex()
     elif kind=='nonfinite':p['normalized_bound_bits']=struct.pack('>d',float('inf')).hex()
     elif kind=='zero':p['multipliers'][0][1]='0000000000000000'
     else:p['multipliers'][0][1]=struct.pack('>d',-struct.unpack('>d',bytes.fromhex(p['multipliers'][0][1]))[0]).hex()
     rejected=validate_cut_proofs(model,bad);assert rejected['valid']is False,(case,kind,rejected);tampered+=1
   record=dict(adaptive_resource_repair=r['adaptive_resource_repair'],zero_phase_one_skips=r['lp_zero_phase_one_skips'],case=case,maximize=maximize,label=label,objective=r['objective'],optimum=opt,status=r['status'],proof_valid=True,point_valid=True,has_master_proof=r['row_generation'].get('bound_proof')is not None,attempts=r['resource_primal']['calls'],successes=int(r['resource_primal']['calls']>0),row_generation=r['row_generation'],resource_primal=r['resource_primal']);records.append(record);atomic_json(dest/'validation.json',record);atomic_json(out/'progress.json',dict(outputs=len(records),valid=len(records),tampered_rejected=tampered,master_proofs=sum(r['has_master_proof']for r in records),resource_primal_calls=sum(r['attempts']for r in records),outputs_using_resource_primal=sum(r['successes']for r in records)))
assert sum(r['successes']for r in records)==400;assert sum(r['has_master_proof']for r in records)>0;atomic_json(out/'records.json',records);atomic_json(out/'summary.json',dict(outputs=len(records),valid=len(records),tampered_rejected=tampered,master_proofs=sum(r['has_master_proof']for r in records),resource_primal_calls=sum(r['attempts']for r in records),outputs_using_resource_primal=sum(r['successes']for r in records)));atomic_json(out/'exit.json',dict(status='complete'));print(json.loads((out/'summary.json').read_text()))
