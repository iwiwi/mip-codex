"""三つの一変数反例を元MPSのFractionで検査。主ソルバーでは使用しない。"""
import json,gzip
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
import subprocess,sys
sys.path.insert(0,str(Path(__file__).resolve().parents[1]/'scripts'))
from verify_cover_cuts import exact_model
from benchmark_common import sha256
root=Path(__file__).resolve().parent
records=[]
for stem in ['cover-duplicate-cancellation','cover-duplicate-objective','cover-decimal-objective']:
 model=root/(stem+'.mps')
 names,variables,rows=exact_model(model);assert names==['x']
 section='';cost=Fraction(0)
 for line in model.read_text().splitlines():
  t=line.split()
  if not line[0].isspace():section=t[0];continue
  if section=='COLUMNS':
   for k in range(1,len(t),2):
    if t[k]=='obj':cost+=Fraction(t[k+1])
 lo=variables['x']['lower'];up=variables['x']['upper']
 for row in rows:
  a=row['coefficients'].get('x',0)
  if a:
   assert a>0
   if row['lower'] is not None:lo=max(lo,row['lower']/a)
   if row['upper'] is not None:up=row['upper']/a if up is None else min(up,row['upper']/a)
 expected=cost*(up if cost<0 else lo)
 out=dict(name=stem,input_sha256=sha256(model),exact_objective_coefficient=str(cost),expected_optimum=str(expected))
 for suffix in ['before','after']:
  path=root/(stem+'-'+suffix+'.json')
  if stem=='cover-duplicate-cancellation' and suffix=='before':path=root/'cover-duplicate-before.json'
  if suffix=='after' and not path.exists():
   path.write_bytes(subprocess.check_output(['taskset','-c','32',str(root.parent/'target/release/mip-codex'),'solve',str(model),'1']))
  r=json.loads(path.read_text(),parse_float=Decimal)
  feasible=False;objective_valid=False;optimal=False
  if r.get('values') is not None:
   x=Fraction(r['values']['x']);feasible=(lo is None or x>=lo) and (up is None or x<=up)
   objective_valid=cost*x==Fraction(r['objective'])
   optimal=feasible and objective_valid and cost*x==expected and r['status']=='optimal'
  out[suffix]=dict(status=r['status'],primal_feasible=feasible,objective_valid=objective_valid,optimal_claim_correct=optimal,result_sha256=sha256(path))
 assert not out['before']['optimal_claim_correct'] and out['after']['optimal_claim_correct']
 records.append(out)
(root/'cover-counterexample-audit.json').write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps(records,indent=2))
