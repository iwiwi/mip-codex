from pathlib import Path
from fractions import Fraction as F
import sys,json,struct
sys.path.insert(0,'scripts');from verify_cover_cuts import exact_model
from benchmark_common import sha256
r=Path.cwd();o=r/'artifacts/priced-row-reduced-cost-diagnostic';o.mkdir();records=[]
for name in ['n3div36.mps.gz','sp97ar.mps.gz','sp98ar.mps.gz','fast0507.mps.gz','eil33-2.mps.gz']:
 p=r/'artifacts/priced-row-subgradient-v1-paired-41'/name/'after/result.json'
 if not p.exists():p=r/'artifacts/priced-row-subgradient-development-screen/cases'/name/'after/result.json'
 v=json.load(open(p));proof=v.get('row_subgradient_proof');assert proof is not None;model=r/'data/raw/benchmark'/name;names,variables,rows,c,offset=exact_model(model,True);point={n:F(str(v['values'][n]))for n in names};valid=all(x in [0,1]for x in point.values());valid=valid and all((row['lower']is None or sum(a*point[n]for n,a in row['coefficients'].items())>=row['lower'])and(row['upper']is None or sum(a*point[n]for n,a in row['coefficients'].items())<=row['upper'])for row in rows);record=dict(name=name,source=str(p),source_sha256=sha256(p),model_sha256=sha256(model),exact_integer_primal_valid=valid)
 if valid:
  reduced={n:c.get(n,F(0))for n in names};rhs=offset
  for i,bits in proof['multipliers']:
   y=F.from_float(struct.unpack('>d',bytes.fromhex(bits))[0]);assert y>=0;row=rows[i];rhs+=y*row['lower']
   for n,a in row['coefficients'].items():reduced[n]-=y*a
  lower=rhs+sum(min(F(0),a)for a in reduced.values());upper=offset+sum(c.get(n,F(0))*point[n]for n in names);fixings=[]
  for n,a in reduced.items():
   if a and lower+abs(a)>upper:
    fixed=0 if a>0 else 1;assert point[n]==fixed;fixings.append(dict(variable=n,value=fixed,opposite_lower=str(lower+abs(a))))
  record.update(exact_lagrangian=str(lower),exact_primal=str(upper),fixings=len(fixings),variables=len(names),fixing_proofs=fixings)
 d=o/name;d.mkdir();(d/'record.json').write_text(json.dumps(record,indent=2)+'\n');records.append({k:v for k,v in record.items()if k!='fixing_proofs'});(o/'summary.json').write_text(json.dumps(records,indent=2)+'\n')
(o/'exit.json').write_text(json.dumps(dict(status='complete'))+'\n');print(records)
