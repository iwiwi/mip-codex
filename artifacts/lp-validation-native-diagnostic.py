import os,json,math,time
from pathlib import Path
from fractions import Fraction
import highspy,numpy as np
os.sched_setaffinity(0,{3})
out=Path('artifacts/lp-validation-native-diagnostic');out.mkdir(exist_ok=True)
for f in sorted(Path('artifacts/lp-validation-dumps').glob('*.json')):
 d=json.load(open(f));p=d['problem'];h=highspy.Highs();h.setOptionValue('output_flag',False);h.setOptionValue('threads',1);h.setOptionValue('time_limit',10.);h.setOptionValue('primal_feasibility_tolerance',1e-9);h.setOptionValue('small_matrix_value',1e-12)
 lp=highspy.HighsLp();lp.num_col_=len(p['columns']);lp.num_row_=len(p['rhs']);lp.col_cost_=[0.]*lp.num_col_;lp.col_lower_=[0.]*lp.num_col_;lp.col_upper_=[highspy.kHighsInf if v is None else v for v in p['upper']];lp.row_upper_=p['rhs'];lp.row_lower_=[b if eq else -highspy.kHighsInf if w is None else b-w for b,eq,w in zip(p['rhs'],p['equality'],p['slack_upper'])]
 lp.a_matrix_.format_=highspy.MatrixFormat.kColwise;starts=[0];index=[];value=[]
 for col in p['columns']:
  for i,a in col:index.append(i);value.append(a)
  starts.append(len(index))
 lp.a_matrix_.start_=starts;lp.a_matrix_.index_=index;lp.a_matrix_.value_=value;h.passModel(lp);h.run();s=h.getSolution();result=dict(source=str(f),model_status=str(h.getModelStatus()),value_valid=s.value_valid,options=dict(threads=1,time_limit=10,objective='zero feasibility diagnostic',primal_feasibility_tolerance=1e-9,small_matrix_value=1e-12),highs_version=h.version())
 if s.value_valid:
  x=s.col_value;exact=[Fraction.from_float(v)for v in x];activity=[Fraction(0)for _ in p['rhs']]
  for j,col in enumerate(p['columns']):
   for i,a in col:activity[i]+=Fraction.from_float(a)*exact[j]
  violation=Fraction(0)
  for i,b in enumerate(p['rhs']):
   residual=Fraction.from_float(b)-activity[i]
   if p['equality'][i]:violation=max(violation,abs(residual))
   else:violation=max(violation,-residual, residual-Fraction.from_float(p['slack_upper'][i]) if p['slack_upper'][i]is not None else Fraction(0))
  bound=max([Fraction(0)]+[-v for v in exact]+[v-Fraction.from_float(u) for v,u in zip(exact,p['upper'])if u is not None]);result.update(point=x,exact_row_violation=str(violation),exact_bound_violation=str(bound),point_absolute_1e6_valid=max(violation,bound)<=Fraction(1,10**6))
 (out/f.name).write_text(json.dumps(result,indent=2)+'\n');print(f.name,result['model_status'],result.get('point_absolute_1e6_valid'),result.get('exact_row_violation'),flush=True)
