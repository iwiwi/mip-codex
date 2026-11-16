#!/usr/bin/env python3
"""原十進MPSと任意f64乗数のLagrangian下界をFractionで計算する。最適化はしない。"""
from fractions import Fraction
import json,math,random
from pathlib import Path
rng=random.Random(2026111502)
values=['0','1','-1','2','-3','0.1','-0.2','1.000000000000000000000001','10000000000000001','1e-300','-1e-300']
cases=[]
for case in range(1000):
    n=rng.randint(1,6);m=rng.randint(0,5)
    costs=[rng.choice(values) for _ in range(n)];offset_rhs=rng.choice(values)
    coefficients=[[rng.choice(values)for _ in range(n)]for _ in range(m)]
    kinds=[rng.choice(['L','G','E'])for _ in range(m)]
    rhs=[rng.choice(values)for _ in range(m)]
    bounds=[]
    for _ in range(n):
        lo=rng.choice([None,-2,0,1]);up=rng.choice([None,2,3,4]);bounds.append((lo,up))
    y=[rng.choice([-2.,-1.,0.,1.,2.,0.1,-0.2,1./3.,1.0000000000000002])for _ in range(m)]
    if case%5==0:y=[0.]*m
    text=['NAME bound','ROWS',' N obj']+[f' {kind} r{i}'for i,kind in enumerate(kinds)]+['COLUMNS']
    for j in range(n):
        text.append(f' x{j} obj {costs[j]}')
        for i in range(m):text.append(f' x{j} r{i} {coefficients[i][j]}')
    text+=['RHS',f' rhs obj {offset_rhs}']+[f' rhs r{i} {rhs[i]}'for i in range(m)]+['BOUNDS']
    for j,(lo,up)in enumerate(bounds):
        if lo is None and up is None:text.append(f' FR b x{j}')
        else:
            text.append(f' MI b x{j}'if lo is None else f' LO b x{j} {lo}')
            if up is not None:text.append(f' UP b x{j} {up}')
    text+=['ENDATA'];text='\n'.join(text)+'\n'
    chosen=[];constant=-Fraction(offset_rhs)
    for i,kind in enumerate(kinds):
        bound=Fraction(rhs[i])if (y[i]>0 and kind in ['G','E'])or(y[i]<0 and kind in ['L','E'])else None
        multiplier=Fraction.from_float(y[i])if bound is not None else Fraction(0)
        chosen.append(multiplier)
        if bound is not None:constant+=multiplier*bound
    expected=constant
    for j,(lo,up)in enumerate(bounds):
        r=Fraction(costs[j])-sum(chosen[i]*Fraction(coefficients[i][j])for i in range(m))
        if (lo is None and r>0)or(up is None and r<0):expected=None;break
        if not r:continue
        expected+=min(r*b for b in [lo,up]if b is not None)
    if expected is not None:
        floor=float(expected)
        if Fraction.from_float(floor)>expected:floor=math.nextafter(floor,-math.inf)
    else:floor=None
    cases.append(dict(mps=text,multipliers=y,lower=[lo for lo,up in bounds],upper=[up for lo,up in bounds],expected_lower=floor))
out=Path(__file__).resolve().parents[1]/'tests/fixtures/original-bound.json'
out.write_text(json.dumps(cases,separators=(',',':'))+'\n')
print(json.dumps(dict(cases=len(cases),finite=sum(c['expected_lower']is not None for c in cases))))
