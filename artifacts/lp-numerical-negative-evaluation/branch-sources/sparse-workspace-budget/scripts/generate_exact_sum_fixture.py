#!/usr/bin/env python3
"""整数累算の一度丸めをPython Fractionで独立照合するfixture。"""
import json,random,struct
from fractions import Fraction
from pathlib import Path
bits=lambda x:struct.unpack('>Q',struct.pack('>d',x))[0]
value=lambda x:struct.unpack('>d',struct.pack('>Q',x))[0]
maximum=value(0x7fefffffffffffff)
cases=[[1e16,1.,-1e16],[maximum,maximum,-maximum],[maximum,maximum],[1.,2.**-53],[1.,2.**-53,2.**-1074],[1.+2.**-52,2.**-53],[2.**-1022,-value(1)],[value(1),value(1),-value(1)]]
rng=random.Random(202611151414)
for _ in range(200):
 terms=[]
 for _ in range(rng.randint(2,80)):
  b=rng.getrandbits(64)
  if b>>52&0x7ff==0x7ff:b^=1<<52
  terms.append(value(b))
 if rng.random()<.6:terms+=[-x for x in terms[:-1]]
 cases.append(terms)
out=[]
for terms in cases:
 exact=sum(map(Fraction,terms),Fraction(0))
 try:expected=bits(float(exact))
 except OverflowError:expected=None
 for _ in range(3):
  rng.shuffle(terms);out.append(dict(terms=list(map(bits,terms)),expected=expected))
Path('tests/fixtures/exact-sum.json').write_text(json.dumps(out,separators=(',',':'))+'\n')
print(len(out))
