from pathlib import Path
import os,sys,json,subprocess,time,resource,shutil
from fractions import Fraction
root=Path.cwd();os.sched_setaffinity(0,{43});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
out=root/'artifacts/numerical-cold-replay';out.mkdir();frozen=root/'artifacts/numerical-cold-replay-frozen';(frozen/'bin').mkdir(parents=True);shutil.copy2(root/'target/release/examples/lp_failure_replay',frozen/'bin/profile');shutil.copy2(root/'examples/lp_failure_replay.rs',frozen/'lp_failure_replay.rs');atomic_json(frozen/'SHA256.json',{str(p.relative_to(frozen)):sha256(p)for p in frozen.rglob('*')if p.is_file()});inputs=sorted((root/'artifacts/numerical-cold-failure-dumps').glob('*/lp.*.json'));assert len(inputs)==4;atomic_json(out/'experiment.json',dict(cpu=43,threads=1,memory_bytes=16*1024**3,seconds=15,profile_sha256=sha256(frozen/'bin/profile'),lib_source_manifest=str(root/'artifacts/numerical-cold-frozen/SHA256.json'),lib_source_manifest_sha256=sha256(root/'artifacts/numerical-cold-frozen/SHA256.json'),inputs=[dict(path=str(p),sha256=sha256(p))for p in inputs],modes=['default','no_singleton','no_skip','no_singleton_no_skip'],raw_recovery_disabled=True,original_mip_claim=False));records=[]
def exact_point(p,r):
 x=r.get('point')
 if x is None:return None
 f=lambda v:Fraction.from_float(float(v));x=list(map(f,x));assert len(x)==len(p['costs']);res=list(map(f,p['rhs']));bound=Fraction(0)
 for j,col in enumerate(p['columns']):
  bound=max(bound,-x[j])
  if p['upper'][j]is not None:bound=max(bound,x[j]-f(p['upper'][j]))
  for i,a in col:res[i]-=f(a)*x[j]
 row=Fraction(0)
 for i,v in enumerate(res):
  row=max(row,abs(v)if p['equality'][i]else -v)
  if not p['equality'][i]and p['slack_upper'][i]is not None:row=max(row,v-f(p['slack_upper'][i]))
 objective=sum((f(c)*v for c,v in zip(p['costs'],x)),Fraction(0));error=abs(objective-f(r['objective']));return dict(valid=max(row,bound)<=Fraction(1,10**6)and error<=Fraction(1,10**8)*(1+abs(objective)),row_violation=str(row),bound_violation=str(bound),objective=str(objective),objective_error=str(error),arithmetic='exact binary64 rational; isolated LP only')
for case,path in enumerate(inputs):
 saved=json.load(open(path));p=saved['problem'];modes=[('default','1','1'),('no_singleton','0','1'),('no_skip','1','0'),('no_singleton_no_skip','0','0')]
 if case%2:modes.reverse()
 for mode,crash,skip in modes:
  d=out/f'{case:02d}'/mode;d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_SINGLETON_CRASH=crash,MIP_LP_ZERO_PHASE_ONE=skip,MIP_LP_NUMERICAL_RECOVERY='0',MIP_LP_TRACE='1');cmd=[str(frozen/'bin/profile'),str(path),'15'];atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(path),environment={k:v for k,v in env.items()if k.startswith('MIP_')}));r=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=75);(d/'stdout.log').write_text(r.stdout);(d/'stderr.log').write_text(r.stderr);assert r.returncode==0,r.stderr;v=json.loads(r.stdout);atomic_json(d/'result.json',v)
  try:validation=exact_point(p,v)
  except Exception as error:validation=dict(valid=None,error=f'{type(error).__name__}: {error}')
  atomic_json(d/'point-validation.json',validation);record=dict(case=case,input=str(path),event=saved['event'],mode=mode,**{k:v.get(k)for k in ['status','objective','dual_bound','seconds','pivots','base_fingerprint']},point_validation=validation);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=16,original_mip_claim=False))
