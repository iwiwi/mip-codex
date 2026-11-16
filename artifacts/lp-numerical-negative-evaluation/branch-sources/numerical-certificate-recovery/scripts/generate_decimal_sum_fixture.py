#!/usr/bin/env python3
"""元十進重複項の合計をFractionで独立生成。"""
from fractions import Fraction
from pathlib import Path
import json,math,random,struct
rng=random.Random(202611151415)
bits=lambda x:struct.unpack('>Q',struct.pack('>d',x))[0]
cases=[['-0.1','-0.2','0.3'],['1.000000000000000000001','-1'],['1e308','1e308'],['1e-300','-9.999999999999999999999999999999999999e-301']]
for _ in range(200):
 e=rng.randint(-250,250);a=rng.randint(1,10**20);small=rng.randint(1,9)
 terms=[f'{a}e{e}',f'{small}e{e-rng.randint(1,70)}',f'-{a}e{e}']
 if all(math.isfinite(float(t)) and float(t)!=0 for t in terms):cases.append(terms)
out=[]
for terms in cases:
 exact=sum((Fraction(t) for t in terms),Fraction(0))
 try:
  value=float(exact);expected=bits(value) if math.isfinite(value) and (value!=0 or exact==0) else None
 except OverflowError:expected=None
 for _ in range(3):
  rng.shuffle(terms);out.append(dict(terms=[t.replace('e','D') if rng.random()<.3 else t for t in terms],expected=expected))
Path('tests/fixtures/decimal-sum.json').write_text(json.dumps(out,separators=(',',':'))+'\n')
print(len(out))
