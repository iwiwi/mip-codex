"""Reverse COLUMNS groups only; verify exact MPS equivalence before own Rust LP."""
import gzip,json,os,resource,subprocess,sys,time
from collections import OrderedDict
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark import verify_snapshot
from benchmark_common import atomic_json,sha256,require_solver_origin
from verify_cover_cuts import exact_model
root=Path.cwd();mode=sys.argv[1];part=int(sys.argv[2]);cpu=int(sys.argv[3]);assert mode in ['original','reverse'];os.sched_setaffinity(0,set(range(37,42)))
snap=root/'artifacts/runs/aggregate-cuts-final-20261115';config=json.loads((snap/'config.json').read_text());verify_snapshot(snap,config);binary=root/'target/release/mip-codex';info=require_solver_origin(root,binary)
cases=sorted(p.parent for p in Path('artifacts/development-root-survey-20261115').glob('*/summary.json')if json.loads(p.read_text()).get('status')=='numerical_error');assert len(cases)==9
cases=[p for p in cases if p.name=='germanrr'];assert len(cases)==1
records=[]
for i,source in enumerate(cases):
 if i%2!=part:continue
 meta=json.loads((source/'experiment.json').read_text());assert meta['split']=='development';original=Path(meta['command'][2]);assert sha256(original)==meta['input_sha256']
 d=root/'artifacts/root-refinement-final-root60'/mode/source.name;d.mkdir(parents=True,exist_ok=True);model=original;equivalence=None
 if mode=='reverse':
  opener=gzip.open if str(original).endswith('.gz')else open
  with opener(original,'rt')as stream:lines=stream.readlines()
  start=next(k for k,l in enumerate(lines)if l.strip()=='COLUMNS')
  end=next(k for k in range(start+1,len(lines))if lines[k].strip()and not lines[k][0].isspace()and not lines[k].startswith('*'))
  groups=OrderedDict();integer={};state=False;comments=[]
  for line in lines[start+1:end]:
   if not line.strip()or line.startswith('*'):comments.append(line);continue
   tokens=line.split()
   if len(tokens)>=3 and tokens[1].strip("'")=='MARKER':state=tokens[2].strip("'")=='INTORG';continue
   name=tokens[0];groups.setdefault(name,[]).append(line);integer[name]=integer.get(name,False)or state
  block=comments[:];state=False;serial=0
  for name in reversed(groups):
   if state!=integer[name]:
    state=integer[name];tag='INTORG'if state else'INTEND';block.append(f"    __revmark{serial} 'MARKER' '{tag}'\n");serial+=1
   block.extend(groups[name])
  if state:block.append(f"    __revmark{serial} 'MARKER' 'INTEND'\n")
  model=d/'reverse.mps';model.write_text(''.join(lines[:start+1]+block+lines[end:]))
  a=exact_model(original,True);b=exact_model(model,True);assert a[0]==list(reversed(b[0]))and set(a[0])==set(b[0])
  assert a[2:]==b[2:]
  assert all(all(a[1][n][k]==b[1][n][k]for k in ['integer','lower','upper'])for n in a[0]);equivalence=True
 env=dict(os.environ,OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',**meta['environment']);env.update(MIP_LP_TRACE='1',MIP_LP_NUMERICAL_DUMP=str(d/'failure'),MIP_ROOT_LP_DUMP=str(d/'root-lp.json'))
 command=[str(binary),'solve',str(model),'60','1'];atomic_json(d/'experiment.json',dict(meta,command=command,purpose='ratio residual refinement diagnostic',build_info=info,binary_sha256=sha256(binary),original_model=str(original),original_sha256=sha256(original),reordered_sha256=sha256(model),exact_equivalence=equivalence,mode=mode,cpu=cpu,environment={k:v for k,v in env.items()if k.startswith('MIP_')}))
 def restrict():
  os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
 start=time.monotonic()
 with(d/'result.json').open('w')as out,(d/'stderr.log').open('w')as err:
  try:p=subprocess.run(command,env=env,preexec_fn=restrict,stdout=out,stderr=err,timeout=120);execution=dict(exit_code=p.returncode)
  except subprocess.TimeoutExpired:execution=dict(external_watchdog=True)
 execution['wall_seconds']=time.monotonic()-start;atomic_json(d/'exit.json',execution)
 record=dict(name=source.name,mode=mode,exact_equivalence=equivalence,execution=execution,point_dump=(d/'root-lp.json').is_file())
 try:
  result=json.loads((d/'result.json').read_text());record.update({k:result.get(k)for k in ['status','objective','dual_bound','nodes','seconds','lp_pivots','lp_refactors','lp_refinement_attempts','lp_refinement_steps','lp_refinement_successes']})
  if record['point_dump']:
   data=json.loads((d/'root-lp.json').read_text(),parse_float=Decimal);names,variables,rows,cost,offset=exact_model(original,True);point={n:Fraction(v)for n,v in zip(data['names'],data['point'])};assert set(point)==set(names)
   bound=Fraction(0);row_error=Fraction(0)
   for name,v in variables.items():
    if v['lower']is not None:bound=max(bound,v['lower']-point[name])
    if v['upper']is not None:bound=max(bound,point[name]-v['upper'])
   for row in rows:
    activity=sum((a*point[n]for n,a in row['coefficients'].items()),Fraction(0))
    if row['lower']is not None:row_error=max(row_error,row['lower']-activity)
    if row['upper']is not None:row_error=max(row_error,activity-row['upper'])
   record.update(lp_bound_violation=str(bound),lp_row_violation=str(row_error),lp_valid=bound<=Fraction(1,10**6)and row_error<=Fraction(1,10**6),lp_objective=str(offset+sum((a*point[n]for n,a in cost.items()),Fraction(0))))
 except Exception as error:record['error']=f'{type(error).__name__}: {error}'
 atomic_json(d/'summary.json',record);records.append(record);print(json.dumps(record),flush=True)
atomic_json(root/f'artifacts/root-refinement-final60-{mode}-{part}-summary.json',records)
