#!/usr/bin/env python3
"""元f64のbitから積和をFractionで計算し最近接/上下界/残差を作る。"""
from fractions import Fraction
import hashlib,json,math,random,struct
from pathlib import Path
rng=random.Random(2026111519)
def bits(v):return struct.unpack('>Q',struct.pack('>d',v))[0]
def random_float():
    while True:
        b=rng.getrandbits(64)
        if (b>>52)&2047!=2047:return struct.unpack('>d',struct.pack('>Q',b))[0]
cases=[]
for case in range(1200):
    mode=case%4
    if mode==0:terms=[(random_float(),random_float())for _ in range(rng.randint(1,12))]
    elif mode==1:
        a,b=random_float(),random_float();terms=[(a,b),(-a,b),*[(rng.uniform(-2,2),rng.uniform(-2,2))for _ in range(3)]]
    elif mode==2:
        a=math.ldexp(float(rng.randint(1,100)),rng.randint(30,900));terms=[(a,1.),(rng.choice([1.,-1.,.5,1e-300]),1.),(-a,1.)]
    else:terms=[(math.ldexp(rng.uniform(-2,2),rng.randint(-30,30)),math.ldexp(rng.uniform(-2,2),rng.randint(-30,30)))for _ in range(rng.randint(1,20))]
    exact=sum((Fraction.from_float(a)*Fraction.from_float(b)for a,b in terms),Fraction(0))
    try:value=float(exact)
    except OverflowError:value=None
    bounds=None;error=None
    if value is not None and math.isfinite(value):
        rounded=Fraction.from_float(value);lo=math.nextafter(value,-math.inf)if rounded>exact else value;hi=math.nextafter(value,math.inf)if rounded<exact else value
        if math.isfinite(lo)and math.isfinite(hi):bounds=[bits(lo),bits(hi)]
        residue=abs(exact-rounded);error=bits(math.nextafter(float(residue),math.inf)if residue else 0.)
    else:value=None
    cases.append(dict(terms=[[bits(a),bits(b)]for a,b in terms],nearest=None if value is None else bits(value),bounds=bounds,error=error))
path=Path(__file__).resolve().parents[1]/'tests/fixtures/exact-dot-values.json';path.write_text(json.dumps(cases,separators=(',',':'))+'\n');print(json.dumps(dict(cases=len(cases),sha256=hashlib.sha256(path.read_bytes()).hexdigest())))
