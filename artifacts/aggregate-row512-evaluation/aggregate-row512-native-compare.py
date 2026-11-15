"""保存証明を元MPSから独立再計算してから連続LPだけを比較。重みは戻さない。"""
import json,os,sys,time
from collections import defaultdict
from fractions import Fraction
from pathlib import Path
import highspy,numpy as np
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model,validate_cut_proofs
from validate_solution import original_lp
from benchmark_common import atomic_json
os.sched_setaffinity(0,{3});root=Path.cwd();out=root/'artifacts/aggregate-row512-native-compare';out.mkdir(exist_ok=True)
smoke=Path(Path('artifacts/aggregate-cuts-initial-after.log').read_text().strip().splitlines()[-1]);records=[]
for name in ['fhnw-binpack4-4','timtab1']:
 model=root/'data/raw/benchmark'/(name+'.mps.gz');names,variables,rows,_,_=exact_model(model,True)
 data=json.loads(Path('artifacts/aggregate-row512-diagnosis',name+'.json').read_text());cuts=data['best']
 for cut in cuts:
  coeff=defaultdict(Fraction);rhs=Fraction(0);assert type(cut['divisor'])is int and 2<=cut['divisor']<=2**52
  for term in cut['proof']:
   assert type(term['weight'])is int and term['weight']>0 and type(term['upper_side'])is bool
   source=rows[term['source_row']];assert term['source_name']==source['name'];w=term['weight']*(1 if term['upper_side']else-1)
   bound=source['upper'if term['upper_side']else'lower'];assert bound is not None;rhs+=w*bound
   for var,a in source['coefficients'].items():coeff[var]+=w*a
  expected=[];projected=[]
  for j,var in enumerate(names):
   a=coeff[var]
   if not a:continue
   assert a.denominator==1
   v=variables[var]
   if v['integer']and v['lower']is not None and v['lower']>=0:
    c=a//cut['divisor']
    if c:expected.append([j,c])
   else:
    b=v['lower'if a>0 else'upper'];assert b is not None;rhs-=a*b;projected.append([j,int(a),int(b)])
  assert expected==cut['coefficients']and rhs//cut['divisor']==cut['upper']and projected==cut['projection']
 old=json.loads((smoke/(name+'.mps.gz')/'rust/result.json').read_text());assert validate_cut_proofs(model,old)['valid']
 previous=[dict(coefficients=c['row']['coefficients'],upper=c['row']['upper'])for c in old['cut_proofs']]
 for label,extra in [('original',[]),('existing_main_cuts',previous),('existing_plus_aggregate',previous+cuts)]:
  lp=original_lp(model);lp.integrality_=[];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',30);h.passModel(lp)
  starts=[0];index=[];values=[]
  for c in extra:
   for j,a in c['coefficients']:index.append(j);values.append(float(a))
   starts.append(len(index))
  if extra:h.addRows(len(extra),np.full(len(extra),-highspy.kHighsInf),np.array([c['upper']for c in extra],dtype=float),len(index),np.array(starts,dtype=np.int32),np.array(index,dtype=np.int32),np.array(values,dtype=float))
  start=time.monotonic();h.run();record=dict(name=name,variant=label,purpose='comparison-only continuous LP; no native weights/primal returned to Rust',highs_version=h.version(),status=str(h.getModelStatus()),objective=h.getObjectiveValue(),cuts=len(extra),aggregate_proofs_checked=len(cuts),seconds=time.monotonic()-start);records.append(record);print(json.dumps(record),flush=True)
atomic_json(out/'results.json',records)
