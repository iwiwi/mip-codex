"""Independent fractional MIR normal form and comparison-only LPs."""
import json,math,os,resource,sys,time
from fractions import Fraction as F
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model,validate_cut_proofs
from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
name=sys.argv[1];cpu=int(sys.argv[2]);os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));root=Path.cwd();source=root/'artifacts/mir-diagnostic'/(name+'.json');data=json.loads(source.read_text());assert data['split']=='development';model=Path(data['model']);assert sha256(model)==data['input_sha256'];names,variables,rows,_,_=exact_model(model,True);out=root/'artifacts/mir-native'/name;out.mkdir(parents=True,exist_ok=True)
def verify(cut):
 p=cut['proof'];ri=p['source_row'];upper=p['upper_side'];d=p['divisor'];assert type(ri)is int and 0<=ri<len(rows)and type(upper)is bool and type(d)is int and 2<=d<=2**52;row=rows[ri];assert row['name']==p['source_name'];rhs=row['upper'if upper else'lower'];assert rhs is not None;sign=1 if upper else-1;rhs*=sign;original={j:sign*row['coefficients'][n]for j,n in enumerate(names)if row['coefficients'].get(n,0)};bounds=p['bounds'];assert [b['variable']for b in bounds]==sorted(original);transformed=[]
 for b in bounds:
  j=b['variable'];u=b['upper'];v=variables[names[j]];assert type(j)is int and type(u)is bool and type(b['bound'])is int;bound=v['upper'if u else'lower'];assert bound is not None and bound==b['bound'];direction=-1 if u else 1;rhs-=original[j]*bound;transformed.append((j,original[j]*direction/F(d),direction,bound,v['integer']))
 beta=rhs/d;fraction=beta-math.floor(beta);assert 0<fraction<1;h=F(d)*(1-fraction);assert h.denominator==1;expected=[];expected_rhs=F(math.floor(beta))
 for j,a,direction,bound,integer in transformed:
  if integer:
   floor=math.floor(a);coefficient=F(floor)+max(F(0),(a-floor-fraction)/(1-fraction))
  else:coefficient=min(F(0),a)/(1-fraction)
  coefficient*=direction
  if coefficient:expected.append([j,coefficient*h]);expected_rhs+=coefficient*bound
 assert [[j,F(a)]for j,a in cut['coefficients']]==expected
 assert F(cut['upper'])==expected_rhs*h
for c in data['cuts']:verify(c)
atomic_json(out/'proof-audit.json',dict(valid=True,cuts=len(data['cuts']),arithmetic='independent Fraction MIR normal form, before integer scaling',model_sha256=sha256(model),diagnostic_sha256=sha256(source)))
base=[]
for label in ['smoke8','smoke10']:
 run=Path(Path(f'artifacts/four-rounds-current-{label}.log').read_text().strip().splitlines()[-1]);p=run/(name+'.mps.gz')/'rust/result.json'
 if p.is_file():
  r=json.loads(p.read_text());check=validate_cut_proofs(model,r);assert check['valid'];atomic_json(out/'existing-proof-audit.json',check);base=[dict(coefficients=c['row']['coefficients'],upper=c['row']['upper'])for c in r['cut_proofs']]
records=[]
for label,cuts in [('original',[]),('existing',base),('existing_plus_mir',base+data['cuts'])]:
 lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp);starts=[0];indices=[];values=[]
 for c in cuts:
  for j,a in c['coefficients']:indices.append(j);values.append(float(a))
  starts.append(len(indices))
 if cuts:h.addRows(len(cuts),np.full(len(cuts),-highspy.kHighsInf),np.array([c['upper']for c in cuts],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(values,dtype=float))
 begin=time.monotonic();h.run();record=dict(name=name,variant=label,highs_version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-begin,cuts=len(cuts),mir_cuts=len(data['cuts'])if label=='existing_plus_mir'else 0,cpu=cpu,threads=1,memory_bytes=16*1024**3,purpose='comparison-only LP; no native primal or multipliers returned to Rust');records.append(record);print(json.dumps(record),flush=True)
atomic_json(out/'results.json',records)
