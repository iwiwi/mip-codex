"""元MPSの厳密整数2行を集約する診断。最適化ライブラリは呼ばない。"""
import itertools,json,math,os,sys,time
from collections import defaultdict
from fractions import Fraction
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_cover_cuts import exact_model
from benchmark_common import atomic_json,sha256
os.sched_setaffinity(0,{int(sys.argv[1])})
root=Path.cwd();out=root/'artifacts/aggregate-row512-diagnosis';out.mkdir(exist_ok=True)
records=[]
for folder in sorted((root/'artifacts/aggregate-cut-roots').iterdir()):
 if not(folder/'root-lp.json').is_file():continue
 meta=json.loads((folder/'experiment.json').read_text());assert meta['split']=='development'
 model=Path(meta['command'][2]);assert sha256(model)==meta['input_sha256']
 names,variables,rows,cost,offset=exact_model(model,True);index={name:j for j,name in enumerate(names)}
 data=json.loads((folder/'root-lp.json').read_text());assert data['names']==names
 x=data['point'];xf=[Fraction(str(v))for v in x];started=time.monotonic()
 violation=Fraction(0)
 for j,name in enumerate(names):
  v=variables[name]
  if v['lower']is not None:violation=max(violation,v['lower']-xf[j])
  if v['upper']is not None:violation=max(violation,xf[j]-v['upper'])
 oriented=[]
 for i,row in enumerate(rows):
  activity=sum((a*xf[index[name]]for name,a in row['coefficients'].items()),Fraction(0))
  if row['lower']is not None:violation=max(violation,row['lower']-activity)
  if row['upper']is not None:violation=max(violation,activity-row['upper'])
  if len(row['coefficients'])>4096 or any(a.denominator!=1 or abs(a)>2**52 for a in row['coefficients'].values()):continue
  for upper in [True,False]:
   bound=row['upper'if upper else'lower'];sign=1 if upper else-1
   if bound is None or bound.denominator!=1 or abs(bound)>2**52:continue
   a={index[name]:int(sign*c)for name,c in row['coefficients'].items()if c};rhs=int(sign*bound)
   norm=math.sqrt(1+sum(float(c)**2 for c in a.values()));slack=(rhs-math.fsum(c*x[j]for j,c in a.items()))/norm
   oriented.append(dict(source_row=i,source_name=row['name'],upper_side=upper,a=a,rhs=rhs,slack=slack))
 oriented=sorted(oriented,key=lambda r:(r['slack'],r['source_row'],r['upper_side']))[:512]
 def divisors(a):
  values=[2,3,4,5,10];g=0
  for c in a.values():
   b=abs(c);g=math.gcd(g,b)
   if b>=2 and b not in values and len(values)<24:values.append(b)
  if g>=2 and g not in values:values.append(g)
  return values
 def rounding(a,rhs,d):
  coeff=[];projection=[]
  for j,c in sorted(a.items()):
   if not c:continue
   v=variables[names[j]]
   if not(v['integer']and v['lower']is not None and v['lower']>=0):
    b=v['lower'if c>0 else'upper']
    if b is None or b.denominator!=1:return None
    rhs-=c*int(b);projection.append([j,c,int(b)])
   elif c//d:coeff.append((j,c//d))
  if abs(rhs)>2**52:return None
  return (tuple(coeff),rhs//d),projection
 single=set()
 for row in oriented:
  for d in divisors(row['a']):
   r=rounding(row['a'],row['rhs'],d)
   if r:single.add(r[0])
 incidences=defaultdict(list)
 for i,row in enumerate(oriented):
  for j,c in row['a'].items():
   v=variables[names[j]]
   active=(abs(x[j]-round(x[j]))>1e-7)if v['integer']else all(b is None or abs(x[j]-float(b))>1e-7 for b in [v['lower'],v['upper']])
   if active:incidences[j].append((i,c))
 pairs={}
 for j,inc in incidences.items():
  for (a,ca),(b,cb)in itertools.combinations(inc[:16],2):
   if oriented[a]['source_row']==oriented[b]['source_row']:continue
   pair=(min(a,b),max(a,b));pairs.setdefault(pair,set()).add((1,1))
   if ca*cb<0:
    g=math.gcd(abs(ca),abs(cb));wa,wb=abs(cb)//g,abs(ca)//g
    pairs[pair].add((wa,wb)if a<b else(wb,wa))
   if len(pairs)>=4096:break
  if len(pairs)>=4096:break
 candidates={};trials=0;valid_roundings=0
 for (a,b),weights in pairs.items():
  for wa,wb in sorted(weights):
   ra,rb=oriented[a],oriented[b];rhs=wa*ra['rhs']+wb*rb['rhs'];coeff=defaultdict(int)
   for row,w in [(ra,wa),(rb,wb)]:
    for j,c in row['a'].items():coeff[j]+=w*c
   coeff={j:c for j,c in coeff.items()if c}
   if abs(rhs)>2**52 or any(abs(c)>2**52 for c in coeff.values()):continue
   for d in divisors(coeff):
    trials+=1;r=rounding(coeff,rhs,d)
    if r is None:continue
    valid_roundings+=1;key,projection=r
    if key in single or key in candidates:continue
    cut,rhs_cut=key;amount=math.fsum(c*x[j]for j,c in cut)-rhs_cut
    if amount<=1e-6:continue
    efficacy=amount/math.sqrt(1+sum(float(c)**2 for j,c in cut))
    proof=[dict(source_row=row['source_row'],source_name=row['source_name'],upper_side=row['upper_side'],weight=w)for row,w in [(ra,wa),(rb,wb)]]
    candidates[key]=dict(coefficients=list(cut),upper=rhs_cut,proof=proof,divisor=d,projection=projection,violation=amount,efficacy=efficacy)
 best=sorted(candidates.values(),key=lambda r:-r['efficacy'])[:32]
 record=dict(name=folder.name,input_sha256=meta['input_sha256'],purpose='exact source arithmetic diagnostic; LP point is fractional; not optimizer integration',lp_objective=data['objective'],lp_max_violation=str(violation),lp_within_absolute_1e6=violation<=Fraction(1,10**6),eligible_selected=len(oriented),source_pairs=len(pairs),trials=trials,valid_roundings=valid_roundings,single_signatures=len(single),new_violated=len(candidates),best=best,seconds=time.monotonic()-started)
 atomic_json(out/(folder.name+'.json'),record);records.append({k:v for k,v in record.items()if k!='best'});print(json.dumps(records[-1]),flush=True)
atomic_json(out/'summary.json',records)
