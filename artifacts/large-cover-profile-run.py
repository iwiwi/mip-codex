from pathlib import Path
import os,sys,json,shutil,subprocess,time,resource
root=Path.cwd();os.sched_setaffinity(0,{47});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from validate_solution import validate
out=root/'artifacts/large-cover-profile-v1';out.mkdir();spec=json.load(open('artifacts/large-cover-profile-source.json'));assert sha256(root/'target/release/mip-codex')==spec['shared_normal_cli_before_sha256'];binary=out/'profile';shutil.copy2(root/'target/release/examples/large_cover_profile',binary)
for name,h in spec['sources'].items():
 assert sha256(root/name)==h;p=out/'source'/name;p.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(root/name,p)
atomic_json(out/'experiment.json',dict(source=spec,binary_sha256=sha256(binary),git_sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),cpu=47,threads=1,memory_bytes=16*1024**3,target=None,stop_value=None))
model=out/'pairs5000.mps';lines=['NAME PAIRS5000','ROWS',' N obj']+[f' L r{i}'for i in range(2500)]+['COLUMNS']+[f' x{i} obj -1 r{i//2} 1'for i in range(5000)]+['RHS']+[f' rhs r{i} 1'for i in range(2500)]+['BOUNDS']+[f' BV b x{i}'for i in range(5000)]+['ENDATA'];model.write_text('\n'.join(lines)+'\n')
records=[]
for label,model,seconds,mode,iterations in [('pairs5000',model,5,'single',2000),('z26-single',root/'data/raw/exploratory/z26.mps.gz',60,'single',20000000),('z26-reseed',root/'data/raw/exploratory/z26.mps.gz',60,'reseed',20000000)]:
 d=out/label;d.mkdir();env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_DIAGNOSTIC_COVER_INITIAL=str(iterations),MIP_DIAGNOSTIC_COVER_EXTENDED=str(iterations));cmd=[str(binary),str(model),str(seconds),mode,str(d/'result.json')];atomic_json(d/'job.json',dict(command=cmd,input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')}));start=time.monotonic();r=subprocess.run(cmd,capture_output=True,text=True,env=env,timeout=seconds+180);(d/'stdout.log').write_text(r.stdout);(d/'stderr.log').write_text(r.stderr);atomic_json(d/'execution.json',dict(exit_code=r.returncode,wall_seconds=time.monotonic()-start));assert r.returncode==0,r.stderr
 metadata=json.loads(r.stdout);assert metadata['base_production_source_directory']==str(root);assert metadata['base_production_fingerprint']=='e5dfb6151bba081e';point=json.load(open(d/'result.json'));v=validate(model,point);atomic_json(d/'validation.json',v);assert v['valid'],v
 if label=='pairs5000':
  assert point['objective']==-2500 and len(point['values'])==5000;assert all(point['values'][f'x{2*i}']+point['values'][f'x{2*i+1}']<=1 for i in range(2500))
 record=dict(label=label,objective=point['objective'],component_seconds=point['component_seconds'],rounds=len(point['rounds']),gmp_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
assert sha256(root/'target/release/mip-codex')==spec['shared_normal_cli_before_sha256'];atomic_json(out/'exit.json',dict(status='complete',outputs=len(records)))
