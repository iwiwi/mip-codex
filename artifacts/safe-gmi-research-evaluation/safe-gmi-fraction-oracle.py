from pathlib import Path
from fractions import Fraction as F
import json,random,itertools,math,subprocess,os,shutil,time
root=Path.cwd();out=root/'artifacts/safe-gmi-fraction-oracle';out.mkdir();os.sched_setaffinity(0,{42});rng=random.Random(2026111705);base=json.load(open('artifacts/safe-gomory-rust-oracle/input.json'))[:200];algebra=[json.loads(s)for s in Path('artifacts/safe-gomory-algebra-oracle/cases.jsonl').read_text().splitlines()][:200];inputs=[];expected=[];checked=0;start=time.monotonic()
frac=lambda v:v-v.numerator//v.denominator
for v,p in zip(algebra,base):
 n=len(v['lower']);pivot=v['pivot'];others=[j for j in range(n)if j!=pivot];rng.shuffle(others);continuous=set(others[:min(len(others),rng.randint(1,2))]);integer=[j not in continuous for j in range(n)];text='\n'.join(s for s in p['model'].splitlines()if "'MARKER'"not in s)+'\n'
 for j in continuous:text=text.replace(f' LI b x{j} ',f' LO b x{j} ').replace(f' UI b x{j} ',f' UP b x{j} ')
 proof=dict(p['proof'],mixed=True);inputs.append(dict(model=text,proof=proof));A=[sum(w*row[j]for w,row in zip(v['weights'],v['a']))for j in range(n)];M=A[pivot];assert M>0;B=sum(w*b for w,b in zip(v['weights'],v['b']));anchors=[l if d==1 else u for l,u,d in zip(v['lower'],v['upper'],v['directions'])];f0=frac(F(B-sum(a*t for a,t in zip(A,anchors)),M))
 if not f0:expected.append(None);continue
 def g(a,is_integer):
  if not is_integer:return a if a>=0 else -f0*a/(1-f0)
  f=frac(a);return f if f<=f0 else f0*(1-f)/(1-f0)
 C=[d*g(F(d*a,M),is_integer)for d,a,is_integer in zip(v['directions'],A,integer)];D=f0+sum(c*t for c,t in zip(C,anchors))
 for w,row,b in zip(v['weights'],v['a'],v['b']):
  term=g(F(w,M),all(integer[j]or a==0 for j,a in enumerate(row)));D-=term*b
  for j,a in enumerate(row):C[j]-=term*a
 scale=math.lcm(*(c.denominator for c in C),D.denominator);Cs=[int(c*scale)for c in C];Ds=int(D*scale);common=math.gcd(*Cs,Ds);Cs=[c//common for c in Cs];Ds//=common
 if not any(Cs):expected.append(None);continue
 expected.append(dict(coefficients=[[j,c]for j,c in enumerate(Cs)if c],lower=Ds))
 ints=[j for j in range(n)if integer[j]];cont=sorted(continuous)
 for assignment in itertools.product(*(range(v['lower'][j],v['upper'][j]+1)for j in ints)):
  fixed=dict(zip(ints,assignment));constraints=[([F(row[j])for j in cont],F(b-sum(row[j]*x for j,x in fixed.items())))for row,b in zip(v['a'],v['b'])]
  for k,j in enumerate(cont):
   c=[F(0)]*len(cont);c[k]=1;constraints.append((c,F(v['upper'][j])));c=[F(0)]*len(cont);c[k]=-1;constraints.append((c,F(-v['lower'][j])))
  vertices=[]
  if len(cont)==1:
   vertices=[[b/a[0]]for a,b in constraints if a[0]]
  else:
   for (a,b),(c,d)in itertools.combinations(constraints,2):
    determinant=a[0]*c[1]-a[1]*c[0]
    if determinant:vertices.append([(b*c[1]-a[1]*d)/determinant,(a[0]*d-b*c[0])/determinant])
  for vertex in vertices:
   if not all(sum(a*x for a,x in zip(row,vertex))<=b for row,b in constraints):continue
   point=dict(fixed);point.update(zip(cont,vertex));assert sum(c*point[j]for j,c in enumerate(Cs))>=Ds,(v['case'],point,Cs,Ds);checked+=1
(out/'input.json').write_text(json.dumps(inputs)+'\n');(out/'expected.json').write_text(json.dumps(expected)+'\n');(out/'summary.json').write_text(json.dumps(dict(models=len(inputs),mixed_vertices_checked=checked,failures=0,seconds=time.monotonic()-start),indent=2)+'\n');print((out/'summary.json').read_text())
