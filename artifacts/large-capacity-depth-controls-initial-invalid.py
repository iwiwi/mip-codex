from pathlib import Path
import os,sys,json,subprocess,resource,copy
root=Path.cwd();os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from validate_solution import validate;from verify_cover_cuts import validate_cut_proofs
binary=root/'artifacts/large-capacity-default-frozen/bin/mip-codex';origin=json.load(open('artifacts/large-capacity-default-origin.json'));assert sha256(binary)==origin['binary_sha256'];out=root/'artifacts/large-capacity-depth-controls';out.mkdir();records=[]
for kind in ['single-cycle-isolates','many-cycles']:
 n=5100;edges=[(n-5+k,n-5+(k+1)%5)for k in range(5)]if kind=='single-cycle-isolates'else[(5*g+k,5*g+(k+1)%5)for g in range(n//5)for k in range(5)];terms=[[]for _ in range(n)]
 for k,(a,b)in enumerate(edges):terms[a].append(k);terms[b].append(k)
 for maximize in [False,True]:
  d=out/f'{kind}-{int(maximize)}';d.mkdir();model=d/'model.mps';lines=['NAME DEPTH','OBJSENSE',' MAX'if maximize else' MIN','ROWS',' N obj']+[f' L r{k}'for k in range(len(edges))]+['COLUMNS']
  for v in range(n):lines+=[f' x{v} obj {3 if maximize else -3}']+[f' x{v} r{k} 1'for k in terms[v]]
  lines+=['RHS',' rhs obj -7']+[f' rhs r{k} 1'for k in range(len(edges))]+['BOUNDS']+[f' BV b x{v}'for v in range(n)]+['ENDATA'];model.write_text('\n'.join(lines)+'\n');env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update({f'MIP_GRAPH_{name}':'0'for name in ['LOCAL_SEARCH','ROOT_COLORING','ROOT_PREFIX','FRACTIONAL_COVER','INPUT_COVER','SPECTRAL_BOUND']});cmd=[str(binary),'solve',str(model),'30'];atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),origin=origin,environment={k:v for k,v in env.items()if k.startswith('MIP_')}));run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=90);(d/'stdout.log').write_text(run.stdout);(d/'stderr.log').write_text(run.stderr);assert run.returncode==0,run.stderr;r=json.loads(run.stdout);atomic_json(d/'result.json',r);g=r['binary_graph'];expected_best=n-3 if kind=='single-cycle-isolates'else2*n//5;expected_upper=expected_best if kind=='single-cycle-isolates'else3*n//5;assert g['best']==expected_best and g['upper']==expected_upper
  if kind=='single-cycle-isolates':assert r['status']=='optimal'and not g['depth_limit_hit']
  else:assert r['status']=='depth_limit'and g['depth_limit_hit']and not g['complete']and r['nodes']<=512
  v=validate(model,r);atomic_json(d/'validation.json',v);assert v['valid'];p=validate_cut_proofs(model,r);atomic_json(d/'proof.json',p);assert p['valid'];tamper=None
  if kind=='many-cycles':
   bad=copy.deepcopy(r);bad['binary_graph']['depth_limit_hit']=False;tamper=validate_cut_proofs(model,bad);atomic_json(d/'tamper.json',bad);atomic_json(d/'tamper-validation.json',tamper);assert tamper['valid']is False
  record=dict(kind=kind,maximize=maximize,status=r['status'],objective=r['objective'],dual_bound=r['dual_bound'],best=g['best'],upper=g['upper'],nodes=r['nodes'],seconds=r['seconds'],gmp_valid=True,proof_valid=True,tamper_rejected=tamper is not None);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=len(records),gmp_valid=len(records),proof_valid=len(records),depth_limit_cases=2,tamper_rejected=2))
