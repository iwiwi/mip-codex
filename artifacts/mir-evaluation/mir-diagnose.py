"""Exact unary MIR candidates, saved development LP points only."""
import argparse,json,math,os,sys,time
from decimal import Decimal
from fractions import Fraction as F
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
p=argparse.ArgumentParser();p.add_argument('--partition',type=int,required=True);p.add_argument('--cpu',type=int,required=True);args=p.parse_args();os.sched_setaffinity(0,{args.cpu});root=Path.cwd();out=root/'artifacts/mir-diagnostic';out.mkdir(exist_ok=True);folders=sorted(d.parent for d in (root/'artifacts/development-root-survey-20261115').glob('*/root-lp.json'));assert len(folders)==77;records=[]
for k,d in enumerate(folders):
 if k%4!=args.partition:continue
 started=time.monotonic();meta=json.loads((d/'experiment.json').read_text());assert meta['split']=='development';model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256'];data=json.loads((d/'root-lp.json').read_text(),parse_float=Decimal);names,variables,rows,_,_=exact_model(model,True);assert names==data['names'];index={n:j for j,n in enumerate(names)};point=[F(v)for v in data['point']];candidates={};work=0
 for ri,row in enumerate(rows):
  if work>=2_000_000:break
  work+=len(row['coefficients'])
  if len(row['coefficients'])>4096 or any(a.denominator!=1 or abs(a)>2**52 for a in row['coefficients'].values()):continue
  if any(b is not None and(b.denominator!=1 or abs(b)>2**52)for b in [row['lower'],row['upper']]):continue
  for upper in [True,False]:
   b=row['upper'if upper else'lower']
   if b is None:continue
   sign=1 if upper else-1;original=[(index[n],int(a)*sign)for n,a in row['coefficients'].items()if a];original.sort()
   ds=list(dict.fromkeys([2,3,4,5,10]+[abs(a)for j,a in original if 2<=abs(a)<=64]))[:24]
   for orientation in [0,1,2]:
    terms=[];bounds=[];rhs=int(b)*sign;valid=True;integer_terms=0
    for j,a in original:
     v=variables[names[j]];choices=[(u,int(v[key]))for u,key in [(False,'lower'),(True,'upper')]if v[key]is not None and v[key].denominator==1 and abs(v[key])<=2**52]
     if not choices:valid=False;break
     if orientation==0:use_upper,bound=min(choices,key=lambda t:(abs(point[j]-t[1]),t[0]))
     else:use_upper,bound=min(choices,key=lambda t:(t[0]!=(orientation==2),t[0]))
     direction=-1 if use_upper else 1;rhs-=a*bound;terms.append((j,a*direction,direction,bound,v['integer']));bounds.append(dict(variable=j,upper=use_upper,bound=bound));integer_terms+=int(v['integer'])
    if not valid or not integer_terms:continue
    for divisor in ds:
     work+=len(terms)
     if work>=2_000_000:break
     residue=rhs%divisor
     if residue==0:continue
     h=divisor-residue;cut_rhs=h*(rhs//divisor);coeff=[]
     for j,a,direction,bound,is_int in terms:
      q=h*(a//divisor)+max(a%divisor-residue,0)if is_int else min(a,0)
      c=q*direction
      if c:coeff.append([j,c]);cut_rhs+=c*bound
     if not coeff or abs(cut_rhs)>2**52 or any(abs(a)>2**52 for j,a in coeff):continue
     violation=sum((a*point[j]for j,a in coeff),F(0))-cut_rhs
     if violation<=F(1,100000):continue
     key=(tuple(map(tuple,coeff)),cut_rhs);score=float(violation)/math.sqrt(1+sum(a*a for j,a in coeff))
     if key not in candidates:candidates[key]=dict(coefficients=coeff,upper=cut_rhs,score=score,violation=str(violation),proof=dict(source_row=ri,source_name=row['name'],upper_side=upper,divisor=divisor,bounds=bounds))
    if work>=2_000_000:break
   if work>=2_000_000:break
 cuts=sorted(candidates.values(),key=lambda c:(-c['score'],c['coefficients'],c['upper']))[:32];record=dict(name=d.name,split='development',model=str(model),input_sha256=sha256(model),point=str(d/'root-lp.json'),point_sha256=sha256(d/'root-lp.json'),work=work,candidates=len(candidates),cuts=cuts,seconds=time.monotonic()-started,purpose='exact original-row MIR diagnostic; no native optimizer outputs used');atomic_json(out/(d.name+'.json'),record);summary=dict(name=d.name,cuts=len(cuts),candidates=len(candidates),seconds=record['seconds']);records.append(summary);print(json.dumps(summary),flush=True)
atomic_json(out/f'partition-{args.partition}.json',dict(models=len(records),models_with_cuts=sum(r['cuts']>0 for r in records),cuts=sum(r['cuts']for r in records),records=records))
