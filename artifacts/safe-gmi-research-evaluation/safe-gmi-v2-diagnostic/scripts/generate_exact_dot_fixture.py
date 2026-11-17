#!/usr/bin/env python3
"""IEEE f64積和の符号をPython Fractionで独立計算。最適化はしない。"""
from fractions import Fraction
import json
from pathlib import Path
import random
import struct
rng=random.Random(2026111505)
def bits():
    return (rng.randrange(2)<<63)|(rng.randrange(2047)<<52)|rng.getrandbits(52)
def number(bits):return struct.unpack('>d',bits.to_bytes(8,'big'))[0]
records=[]
for case in range(500):
    terms=[[bits(),bits()] for _ in range(rng.randrange(1,12))]
    if case%2:
        terms += [[a^(1<<63),b] for a,b in terms]
        terms += [[bits(),bits()]]
        rng.shuffle(terms)
    total=sum((Fraction.from_float(number(a))*Fraction.from_float(number(b)) for a,b in terms),Fraction())
    records.append(dict(terms=terms,sign=(total>0)-(total<0)))
maximum=0x7fefffffffffffff
terms=[[maximum,maximum]]*1000+[[maximum^(1<<63),maximum]]*1000+[[1,1]]
records.append(dict(terms=terms,sign=1))
Path('tests/fixtures/exact-dot-sign.json').write_text(json.dumps(records,separators=(',',':'))+'\n')
