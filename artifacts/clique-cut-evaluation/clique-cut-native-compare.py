"""Independent pair forcing proofs and comparison-only continuous LPs."""
import itertools,json,math,os,resource,sys,time
from fractions import Fraction as F
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from validate_solution import original_lp
from benchmark_common import atomic_json,sha256
name=sys.argv[1];cpu=int(sys.argv[2]);os.sched_setaffinity(0,{cpu});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));root=Path.cwd();source=root/'artifacts/clique-cut-diagnostic'/(name+'.json');data=json.loads(source.read_text());assert data['split']=='development';model=Path(data['model']);assert sha256(model)==data['input_sha256'];names,variables,rows,_,_=exact_model(model,True);out=root/'artifacts/clique-cut-native'/name;out.mkdir(parents=True,exist_ok=True);audits=[]
for cut in data['cuts']:
 coeff=cut['coefficients'];assert coeff==sorted(coeff)and len({j for j,a in coeff})==len(coeff)and all(a in [-1,1]for j,a in coeff);literals={(j,a<0)for j,a in coeff};assert cut['upper']==1-sum(a<0 for j,a in coeff)
 for j,neg in literals:
  v=variables[names[j]];assert v['integer']and v['lower']is not None and v['upper']is not None and 0<=math.ceil(v['lower'])<=math.floor(v['upper'])<=1
 expected={tuple(sorted(pair))for pair in itertools.combinations(literals,2)};seen=set();pairs=[]
 for proof in cut['pair_proofs']:
  pair=tuple(sorted(tuple(l)for l in proof['literals']));assert len(pair)==2 and pair in expected and pair not in seen;seen.add(pair);ri=proof['source_row'];row=rows[ri];assert row['name']==proof['source_name']and type(proof['upper_side'])is bool;sign=1 if proof['upper_side']else-1;rhs=row['upper'if proof['upper_side']else'lower'];assert rhs is not None;rhs*=sign;forced={names[j]:F(not neg)for j,neg in pair};minimum=F(0)
  for v,a in row['coefficients'].items():
   a*=sign
   if not a:continue
   if v in forced:bound=forced[v]
   else:
    bound=variables[v]['lower'if a>0 else'upper'];assert bound is not None
   minimum+=a*bound
  assert minimum>rhs
  pairs.append(dict(source_row=ri,forced={v:str(b)for v,b in forced.items()},minimum=str(minimum),rhs=str(rhs)))
 assert seen==expected
 audits.append(dict(coefficients=coeff,upper=cut['upper'],pair_checks=pairs,valid=True))
atomic_json(out/'proof-audit.json',dict(valid=True,cuts=len(audits),pair_checks=sum(len(a['pair_checks'])for a in audits),checks=audits,diagnostic_sha256=sha256(source),model_sha256=sha256(model)));records=[]
for label,cuts in [('original',[]),('with_clique_cuts',data['cuts'])]:
 lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',60);h.passModel(lp);starts=[0];indices=[];values=[]
 for cut in cuts:
  for j,a in cut['coefficients']:indices.append(j);values.append(a)
  starts.append(len(indices))
 if cuts:h.addRows(len(cuts),np.full(len(cuts),-highspy.kHighsInf),np.array([c['upper']for c in cuts],dtype=float),len(indices),np.array(starts,dtype=np.int32),np.array(indices,dtype=np.int32),np.array(values,dtype=float))
 begin=time.monotonic();h.run();r=dict(name=name,variant=label,highs_version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),seconds=time.monotonic()-begin,cuts=len(cuts),cpu=cpu,threads=1,memory_bytes=16*1024**3,purpose='comparison-only LP; no native point or multipliers returned to Rust');records.append(r);print(json.dumps(r),flush=True)
atomic_json(out/'results.json',records)
