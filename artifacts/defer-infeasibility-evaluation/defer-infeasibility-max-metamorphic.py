import gzip,json,os,sys,subprocess,resource
from pathlib import Path
root=Path.cwd();sys.path.insert(0,str(root/'scripts'))
from benchmark_common import sha256,atomic_json,require_solver_origin
os.sched_setaffinity(0,set(range(33,42)));out=root/'artifacts/defer-infeasibility-max-metamorphic';out.mkdir(exist_ok=True)
original=root/'data/raw/benchmark/gen-ip002.mps.gz';raw=gzip.open(original,'rt').read().splitlines();section='';objective=None;lines=[]
for line in raw:
 t=line.split()
 if not t or line.startswith('*'):lines.append(line);continue
 if not line[0].isspace():
  section=t[0]
  if section=='ROWS':lines.extend(['OBJSENSE',' MAX'])
  if section=='OBJSENSE':raise ValueError('fixture expects original default MIN')
  lines.append(line);continue
 if section=='ROWS'and t[0]=='N'and objective is None:objective=t[1]
 if section in ['COLUMNS','RHS']and objective is not None:
  for k in range(1,len(t)-1,2):
   if t[k]==objective:t[k+1]=t[k+1][1:]if t[k+1].startswith('-')else'-'+t[k+1]
  line=' '+ ' '.join(t)
 lines.append(line)
model=out/'max-negated.mps';model.write_text('\n'.join(lines)+'\n');binary=root/'target/release/mip-codex'
info=require_solver_origin(root,binary)
def restrict():
 os.sched_setaffinity(0,{0});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for sense,path in [('min',original),('max',model)]:
 d=out/sense;d.mkdir(exist_ok=True);command=['/usr/bin/time','-v','-o',str(d/'resource.log'),str(binary),'solve',str(path),'30','50000']
 atomic_json(d/'experiment.json',dict(command=command,build_info=info,binary_sha256=sha256(binary),input_sha256=sha256(path),original_sha256=sha256(original),kind='metamorphic regression not independent benchmark instance',cpu=0,threads=1,memory_bytes=16*1024**3))
 with(d/'result.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run(command,preexec_fn=restrict,stdout=stream,stderr=err,timeout=75)
 atomic_json(d/'exit.json',dict(exit_code=p.returncode));assert p.returncode==0
 with(d/'validation.log').open('w')as stream:q=subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(path),str(d/'result.json'),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)],stdout=stream,stderr=subprocess.STDOUT)
 assert q.returncode==0 and json.loads((d/'strict-validation.json').read_text())['valid']is True and json.loads((d/'official-validation.json').read_text())['valid']is True
b=json.loads((out/'min/result.json').read_text());a=json.loads((out/'max/result.json').read_text())
for k in ['values','nodes','lp_solves','lp_pivots','status','deferred_infeasibilities','deferred_infeasibilities_pending','lp_search_termination']:assert a[k]==b[k],(k,a[k],b[k])
for k in ['objective','dual_bound','deferred_infeasibility_bound']:assert a[k]==-b[k],(k,a[k],b[k])
assert a['deferred_infeasibilities']>0 and a['status']=='numerical_error'
print(json.dumps(dict(nodes=a['nodes'],max_objective=a['objective'],max_dual=a['dual_bound'],max_deferred_bound=a['deferred_infeasibility_bound'],deferred=a['deferred_infeasibilities'],valid=True)))
