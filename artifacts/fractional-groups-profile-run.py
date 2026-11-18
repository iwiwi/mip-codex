from pathlib import Path
import json,os,sys,subprocess,shutil,time,resource,copy
root=Path.cwd();os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MKL_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256,solver_source_fingerprint;from validate_solution import validate;from verify_cover_cuts import exact_model;from verify_binary_graph import verify
out=root/'artifacts/fractional-groups-profile';out.mkdir();binary=out/'profile';shutil.copy2(root/'target/release/examples/fractional_groups_profile',binary);shutil.copytree(root/'examples/support/fractional_profile',out/'source');shutil.copy2(root/'examples/fractional_groups_profile.rs',out/'profile.rs');shutil.copy2(root/'src/exact.rs',out/'exact.rs');shutil.copytree(root/'scripts',out/'scripts',ignore=shutil.ignore_patterns('__pycache__'));manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};inputs=[('sorrell3.mps.gz',root/'data/raw/benchmark/sorrell3.mps.gz','development'),('sorrell7.mps.gz',root/'data/raw/exploratory/sorrell7.mps.gz','exploratory; outside240')];assert manifest[inputs[0][0]]['split']=='development'and sha256(inputs[0][1])==manifest[inputs[0][0]]['sha256'];assert inputs[1][0]not in manifest;records=[]
atomic_json(out/'experiment.json',dict(cpu=45,threads=1,memory_bytes=16*1024**3,seconds=[0.5,2,10],modes=[0,1],binary_sha256=sha256(binary),profile_source=json.load(open('artifacts/fractional-groups-profile-source.json')),base_production_fingerprint=solver_source_fingerprint(root),git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),purpose='standalone root-bound profile; original-MPS Fraction certificate and zero-point GMP; no independent search-tree claim'))
for name,model,split in inputs:
 exact=exact_model(model,True)
 for seconds in [0.5,2,10]:
  for mode in ([0,1]if seconds!=2 else[1,0]):
   d=out/name/f'{seconds}-{mode}';d.mkdir(parents=True);result=d/'result.json';env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_DIAGNOSTIC_CHEAP_GROUPS=str(mode));cmd=[str(binary),str(model),str(seconds),str(result)];atomic_json(d/'job.json',dict(command=cmd,environment={k:v for k,v in env.items()if k.startswith(('MIP_','OMP_','OPENBLAS_','MKL_'))},input_sha256=sha256(model),split=split));start=time.monotonic();run=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=seconds+60);(d/'stdout.log').write_text(run.stdout);(d/'stderr.log').write_text(run.stderr);atomic_json(d/'execution.json',dict(exit_code=run.returncode,wall_seconds=time.monotonic()-start));assert run.returncode==0,run.stderr;r=json.load(open(result));proof=verify(model,r,exact);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True;v=validate(model,r);atomic_json(d/'point-validation.json',v);assert v['valid']is True
   cert=r['binary_graph']['fractional_cover'].get('certificate');tamper=0
   if cert:
    for kind in ['total','minimum','upper','zero','duplicate']:
     bad=copy.deepcopy(r);c=bad['binary_graph']['fractional_cover']['certificate']
     if kind=='total':c['total']+=1
     elif kind=='minimum':c['minimum']+=1
     elif kind=='upper':c['upper']-=1
     elif kind=='zero':c['counts'][0]=0
     else:c['cliques'][0].append(c['cliques'][0][0])
     try:verify(model,bad,exact)
     except (AssertionError,KeyError,ValueError,TypeError):tamper+=1
     else:raise AssertionError('tampered certificate accepted: '+kind)
   record=dict(name=name,seconds=seconds,mode=mode,dual_bound=r['dual_bound'],coloring_upper=r['binary_graph']['root_coloring_upper'],upper=r['binary_graph']['upper'],component_seconds=r['component_seconds'],stats=r['binary_graph']['fractional_cover']['stats'],proof_valid=True,gmp_valid=True,tamper_rejected=tamper);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'summary.json',dict(outputs=len(records),proof_valid=len(records),gmp_valid=len(records),tamper_rejected=sum(r['tamper_rejected']for r in records)));atomic_json(out/'exit.json',dict(status='complete'));print(json.load(open(out/'summary.json')))
