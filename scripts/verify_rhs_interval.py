#!/usr/bin/env python3
"""Check diagnostic shifted-row relaxations against exact original MPS; no solver calls."""
from fractions import Fraction as F
from pathlib import Path
import json,math,struct,sys
from verify_cover_cuts import exact_model
from benchmark_common import sha256

def binary(text):
    if not isinstance(text,str) or len(text)!=16:raise ValueError('invalid f64 bits')
    v=struct.unpack('>d',bytes.fromhex(text))[0]
    if not math.isfinite(v):raise ValueError('nonfinite binary value')
    return F(v)

def verify(model,dump):
    names,variables,rows=exact_model(model)
    index_by_name={name:i for i,name in enumerate(names)}
    records=[]
    for line in Path(dump).read_text().splitlines():
        p=json.loads(line)
        if set(p)!={'source_row','source_name','upper','rhs_bits','shift_bits'}:raise ValueError('record schema')
        index=p['source_row']
        if type(index)is not int or not 0<=index<len(rows):raise ValueError('row index')
        row=rows[index]
        if row['name']!=p['source_name'] or type(p['upper'])is not bool:raise ValueError('row identity')
        if row['lower']is not None and row['upper']is not None:raise ValueError('equality/range is not a permitted fallback')
        side='upper'if p['upper']else'lower';b=row[side]
        if b is None:raise ValueError('missing original side')
        sign=1 if p['upper']else-1;shift={}
        for j,bits in p['shift_bits']:
            if type(j)is not int or not 0<=j<len(names)or j in shift:raise ValueError('shift index')
            shift[j]=binary(bits)
        required=sign*b
        for name,a in row['coefficients'].items():
            j=index_by_name[name];hat=F(float(a));error=sign*(hat-a)
            required-=sign*hat*shift.get(j,F(0))
            if error:
                v=variables[name];lo=v['lower'];up=v['upper']
                if lo is None or up is None:raise ValueError('uncertain coefficient with unbounded original box')
                required+=max(error*lo,error*up)
        actual=binary(p['rhs_bits'])
        if actual<required:raise ValueError('nonconservative shifted RHS')
        records.append(dict(source_row=index,source_name=row['name'],required=str(required),actual=str(actual),margin=str(actual-required)))
    if not records:raise ValueError('empty diagnostic dump')
    return dict(valid=True,rows=len(records),input_sha256=sha256(model),dump_sha256=sha256(dump),records=records)

if __name__=='__main__':
    try:
        result=verify(Path(sys.argv[1]),Path(sys.argv[2]));print(json.dumps(result,indent=2))
    except Exception as e:
        print(json.dumps(dict(valid=False,error=f'{type(e).__name__}: {e}')));sys.exit(1)
