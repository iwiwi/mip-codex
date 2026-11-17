from pathlib import Path
from fractions import Fraction as F
import random,json,itertools,os,time
r=random.Random(2026111703);out=Path('artifacts/safe-gomory-algebra-oracle');out.mkdir();os.sched_setaffinity(0,{42});checked=0;nontrivial=0;start=time.monotonic()
with(out/'cases.jsonl').open('w')as stream:
 for case in range(500):
  n=r.randint(2,6);m=r.randint(2,8);lower=[r.randint(-3,1)for _ in range(n)];upper=[l+r.randint(1,3)for l in lower];known=[r.randint(l,u)for l,u in zip(lower,upper)];a=[[r.randint(-5,5)for _ in range(n)]for _ in range(m)];b=[sum(c*x for c,x in zip(row,known))+r.randint(0,4)for row in a];weights=[r.randint(-16,16)for _ in range(m)];A=[sum(w*row[j]for w,row in zip(weights,a))for j in range(n)];eligible=[j for j in range(n)if A[j]]
  if not eligible:continue
  k=r.choice(eligible)
  if A[k]<0:weights=[-w for w in weights];A=[-v for v in A]
  M=A[k];B=sum(w*v for w,v in zip(weights,b));directions=[r.choice([-1,1])for _ in range(n)];anchors=[lo if d==1 else up for lo,up,d in zip(lower,upper,directions)];Bp=B-sum(v*t for v,t in zip(A,anchors));R=[(d*v)%M for d,v in zip(directions,A)];S=[w%M for w in weights];R0=Bp%M;C=[directions[j]*R[j]-sum(s*row[j]for s,row in zip(S,a))for j in range(n)];D=R0+sum(d*v*t for d,v,t in zip(directions,R,anchors))-sum(s*v for s,v in zip(S,b));fraction=lambda v:v-(v.numerator//v.denominator);assert all(F(v,M)==fraction(F(d*z,M))for v,d,z in zip(R,directions,A));assert all(F(s,M)==fraction(F(w,M))for s,w in zip(S,weights));assert F(R0,M)==fraction(F(Bp,M));valid_points=0
  for point in itertools.product(*(range(l,u+1)for l,u in zip(lower,upper))):
   activity=[sum(c*x for c,x in zip(row,point))for row in a]
   if any(v>rhs for v,rhs in zip(activity,b)):continue
   assert sum(c*x for c,x in zip(C,point))>=D,(case,point,C,D);slacks=[rhs-v for rhs,v in zip(b,activity)];z=[d*(x-t)for d,x,t in zip(directions,point,anchors)];assert all(v>=0 for v in z+slacks);assert sum(v*x for v,x in zip(R,z))+sum(s*x for s,x in zip(S,slacks))>=R0;valid_points+=1;checked+=1
  assert valid_points>0;nontrivial+=int(R0!=0);stream.write(json.dumps(dict(case=case,a=a,b=b,lower=lower,upper=upper,weights=weights,pivot=k,directions=directions,coefficients=C,rhs=D,denominator=M,feasible_points=valid_points))+'\n')
(out/'summary.json').write_text(json.dumps(dict(seed=2026111703,cases=500,generated=sum(1 for _ in(out/'cases.jsonl').open()),feasible_points_checked=checked,nonzero_fractional_rhs=nontrivial,failures=0,seconds=time.monotonic()-start),indent=2)+'\n');print((out/'summary.json').read_text())
