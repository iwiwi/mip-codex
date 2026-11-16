#!/usr/bin/env python3
"""FMA境界・subnormal・3成分相殺を独立Fractionで検査するbit fixture。"""
from fractions import Fraction
import hashlib,json,math,random,struct
from pathlib import Path
rng=random.Random(2026111514)
def bits(v):return struct.unpack('>Q',struct.pack('>d',v))[0]
def number(v):return struct.unpack('>d',struct.pack('>Q',v))[0]
def normal(e):return math.ldexp(1+rng.getrandbits(52)/2**52,e)*rng.choice([-1,1])
cases=[]
for i in range(2400):
    mode=i%6
    if mode==0:
        e=rng.randint(-900,-68);a=normal(e);b=normal(rng.choice([-972,-971,-970,-969,-968])-e);terms=[(a,b)]
    elif mode==1:
        a=number(rng.randint(1,2**52-1))*rng.choice([-1,1]);b=normal(rng.choice([50,51,52,53,54]));terms=[(a,b)]
    elif mode==2:
        a,b=normal(rng.randint(-700,700)),normal(rng.randint(-700,700));terms=[(a,b),(-a,b),(1.,1.)]
    elif mode==3:
        a,b,c=normal(800),normal(0),normal(-800);terms=[(a,1.),(b,1.),(c,1.),(-a,1.),(-b,1.)]
    elif mode==4:
        a=number(bits(1.)+rng.randrange(32));terms=[(a,1.),(2.**-52,.5)]
    else:
        terms=[(normal(rng.randint(-20,20)),float(rng.randint(-1000000,1000000)))for _ in range(rng.randint(2,30))]
    exact=sum((Fraction(a)*Fraction(b)for a,b in terms),Fraction(0))
    try:v=float(exact)
    except OverflowError:v=None
    bounds=error=None
    if v is not None and math.isfinite(v):
        rounded=Fraction(v);lo=math.nextafter(v,-math.inf)if rounded>exact else v;hi=math.nextafter(v,math.inf)if rounded<exact else v
        if math.isfinite(lo)and math.isfinite(hi):bounds=[bits(lo),bits(hi)]
        residual=abs(exact-rounded);error=bits(math.nextafter(float(residual),math.inf)if residual else 0.)
    else:v=None
    cases.append(dict(terms=[[bits(a),bits(b)]for a,b in terms],nearest=None if v is None else bits(v),bounds=bounds,error=error))
p=Path(__file__).resolve().parents[1]/'tests/fixtures/exact-fma-values.json';p.write_text(json.dumps(cases,separators=(',',':'))+'\n');print(json.dumps(dict(cases=len(cases),sha256=hashlib.sha256(p.read_bytes()).hexdigest())))
