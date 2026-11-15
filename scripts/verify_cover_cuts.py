#!/usr/bin/env python3
"""元MPSをFractionで読み、Rustのcover証明を独立に検査。最適化は実行しない。"""
from decimal import Decimal
from fractions import Fraction
import gzip
import json
import math
from pathlib import Path
import sys
from benchmark_common import atomic_json, sha256

def exact_model(path, include_objective=False):
    rows={}; variables={}; rhs={}; ranges={}; vectors={}; section=''; integer=False; ended=False; objective_name=None
    opener=gzip.open if str(path).endswith('.gz') else open
    number=lambda s:Fraction(s.replace('D','E').replace('d','e'))
    with opener(path,'rt') as stream:
        for line in stream:
            if not line.strip() or line.startswith('*'):continue
            t=[]
            for token in line.split():
                if token.startswith('$'):break
                t.append(token)
            if not t:continue
            if not line[0].isspace():
                section=t[0]
                if section=='OBJNAME' and len(t)==2:objective_name=t[1]
                if section=='ENDATA':ended=True;break
                if section not in ['NAME','ROWS','COLUMNS','RHS','RANGES','BOUNDS','OBJSENSE','OBJNAME']:
                    raise ValueError('unsupported MPS section')
                continue
            if section=='OBJNAME':
                if len(t)!=1:raise ValueError('invalid OBJNAME')
                objective_name=t[0]
            elif section=='ROWS':
                if len(t)!=2 or t[0] not in ['N','L','G','E'] or t[1] in rows:raise ValueError('invalid row')
                rows[t[1]]=dict(kind=t[0],coefficients={})
            elif section=='COLUMNS':
                if len(t)>=3 and t[1].strip("'")=='MARKER':
                    tag=t[2].strip("'")
                    if tag not in ['INTORG','INTEND']:raise ValueError('unsupported marker')
                    integer=tag=='INTORG';continue
                if len(t) not in [3,5]:raise ValueError('unsupported column')
                var=variables.setdefault(t[0],dict(integer=False,lower=Fraction(0),upper=None,explicit=False))
                var['integer'] |= integer
                for k in range(1,len(t),2):
                    coeff=rows[t[k]]['coefficients'];coeff[t[0]]=coeff.get(t[0],Fraction(0))+number(t[k+1])
            elif section in ['RHS','RANGES']:
                if len(t) not in [3,5]:raise ValueError('unsupported vector')
                if section in vectors and vectors[section]!=t[0]:raise ValueError('multiple vectors')
                vectors[section]=t[0];values=rhs if section=='RHS' else ranges
                for k in range(1,len(t),2):values[t[k]]=number(t[k+1])
            elif section=='BOUNDS':
                if len(t) not in [3,4]:raise ValueError('unsupported bounds')
                if 'BOUNDS' in vectors and vectors['BOUNDS']!=t[1]:raise ValueError('multiple bounds')
                vectors['BOUNDS']=t[1]
                var=variables[t[2]];kind=t[0];v=number(t[3]) if len(t)==4 else None
                if kind in ['LO','LI']:
                    if v is None:raise ValueError('missing bound')
                    var['lower']=v
                    if kind=='LI':var['integer']=True
                elif kind in ['UP','UI']:
                    if v is None:raise ValueError('missing bound')
                    if v<0 and not var['explicit']:var['lower']=None
                    var['upper']=v
                    if kind=='UI':var['integer']=True
                elif kind=='FX':
                    if v is None:raise ValueError('missing bound')
                    var['lower']=var['upper']=v
                elif kind=='BV':var.update(lower=Fraction(0),upper=Fraction(1),integer=True)
                elif kind=='FR':var.update(lower=None,upper=None)
                elif kind=='MI':var['lower']=None
                elif kind=='PL':var['upper']=None
                else:raise ValueError('unsupported bound type')
                var['explicit']=True
    if not ended or integer:raise ValueError('truncated MPS')
    for var in variables.values():
        if var['integer'] and not var['explicit']:var['upper']=Fraction(1)
    constraints=[]
    for name,row in rows.items():
        kind=row['kind']
        if kind=='N':continue
        b=rhs.get(name,Fraction(0));lo=b if kind in ['G','E'] else None;up=b if kind in ['L','E'] else None
        if name in ranges:
            r=ranges[name]
            if kind=='L':lo=b-abs(r)
            elif kind=='G':up=b+abs(r)
            elif r>=0:up=b+r
            else:lo=b+r
        constraints.append(dict(name=name,lower=lo,upper=up,coefficients=row['coefficients']))
    if include_objective:
        objective_name=objective_name or next((name for name,row in rows.items() if row['kind']=='N'),None)
        if objective_name not in rows or rows[objective_name]['kind']!='N':raise ValueError('missing objective')
        return list(variables),variables,constraints,rows[objective_name]['coefficients'],-rhs.get(objective_name,Fraction(0))
    return list(variables),variables,constraints

def verify_objective_grid(names,variables,rows,objective,offset,proof):
    if not isinstance(proof,dict) or type(proof.get('step'))!=int or type(proof.get('offset'))!=int:
        raise AssertionError('invalid objective grid schema')
    if offset.denominator!=1 or proof['offset']!=offset:raise AssertionError('objective grid offset mismatch')
    known={name for name,var in variables.items() if var['integer']}
    implications=proof.get('implied')
    if not isinstance(implications,list):raise AssertionError('objective grid implications must be a list')
    for implication in implications:
        j=implication['variable'];i=implication['source_row']
        if type(j)!=int or j<0 or j>=len(names) or type(i)!=int or i<0 or i>=len(rows):raise AssertionError('bad implied-integer index')
        name=names[j];row=rows[i]
        if name in known or implication['source_name']!=row['name']:raise AssertionError('repeated or wrong implied-integer source')
        if row['lower'] is None or row['lower']!=row['upper'] or row['lower'].denominator!=1:raise AssertionError('implied integrality requires an integer equality')
        if abs(row['coefficients'].get(name,Fraction(0)))!=1:raise AssertionError('implied integrality requires unit coefficient')
        for column,a in row['coefficients'].items():
            if a.denominator!=1 or (a and column!=name and column not in known):raise AssertionError('unproved integer term in equality')
        known.add(name)
    step=0
    for name,c in objective.items():
        if not c:continue
        if c.denominator!=1 or name not in known:raise AssertionError('objective has an unproved integer term')
        step=math.gcd(step,abs(c.numerator))
    if step<=0 or proof['step']!=step:raise AssertionError('objective grid gcd mismatch')
    return dict(objective_grid_valid=True,objective_step=step,implied_integer_variables=len(implications))


def validate_cut_proofs(model_path,result):
    cuts=result.get('cut_proofs',[])
    grid=result.get('objective_grid')
    metadata=dict(valid=None,checked=0,arithmetic='Python Fraction from original MPS tokens',checker_sha256=sha256(Path(__file__)))
    if not isinstance(cuts,list):return dict(metadata,valid=False,error='cut_proofs must be a list')
    if not cuts and grid is None:return dict(metadata,valid=True)
    try:names,variables,rows,objective,offset=exact_model(model_path,True)
    except Exception as error:return dict(metadata,error=f'cannot read exact MPS: {type(error).__name__}: {error}')
    indices={name:j for j,name in enumerate(names)}
    try:
        if grid is not None:
            metadata['objective_grid_valid']=False
            metadata.update(verify_objective_grid(names,variables,rows,objective,offset,grid))
        for cut in cuts:
            proof=cut['proof'];i=proof['source_row'];cover=proof['cover'];side=proof['upper_side']
            if type(i)!=int or i<0 or i>=len(rows):raise AssertionError('bad source index')
            source=rows[i]
            if proof['source_name']!=source['name'] or type(side)!=bool:raise AssertionError('bad source row/side')
            if not isinstance(cover,list) or any(type(j)!=int or j<0 or j>=len(names) for j in cover):raise AssertionError('bad variable index')
            if cover!=sorted(set(cover)):raise AssertionError('duplicate or unsorted cover')
            selected={names[j] for j in cover};sign=1 if side else -1
            bound=source['upper' if side else 'lower']
            if bound is None:raise AssertionError('source side missing')
            divisor=proof.get('divisor')
            if divisor is not None:
                if type(divisor)!=int or divisor<2 or divisor>2**52 or cover:raise AssertionError('bad rounding divisor/cover')
                oriented_rhs=sign*bound
                if oriented_rhs.denominator!=1:raise AssertionError('rounding requires an integral source RHS')
                coeff=[]
                for name,a in source['coefficients'].items():
                    if not a:continue
                    var=variables[name]
                    if a.denominator!=1:raise AssertionError('rounding requires integer source coefficients')
                    a*=sign
                    if not (var['integer'] and var['lower'] is not None and var['lower']>=0):
                        projected_bound=var['lower'] if a>0 else var['upper']
                        if projected_bound is None:raise AssertionError('rounding projection requires a finite bound')
                        oriented_rhs-=a*projected_bound
                        continue
                    c=a//divisor
                    if c:coeff.append([indices[name],Fraction(c)])
                coeff.sort(key=lambda p:p[0]);row=cut['row']
                actual=[[j,Fraction(a)] for j,a in row['coefficients']]
                if row['lower'] is not None or Fraction(row['upper'])!=oriented_rhs//divisor or actual!=coeff:
                    raise AssertionError('derived inequality does not match floor division')
                metadata['checked']+=1
                continue
            minimum=Fraction(0);coeff=[];negative=0;found=set()
            for name,a in source['coefficients'].items():
                if a==0:continue
                var=variables[name]
                binary=(var['integer'] and var['lower'] is not None and var['upper'] is not None
                        and math.ceil(var['lower'])>=0 and math.floor(var['upper'])<=1)
                a*=sign
                if not binary:
                    if name in selected:raise AssertionError('cover includes a nonbinary column')
                    column_bound=var['lower'] if a>0 else var['upper']
                    if column_bound is None:raise AssertionError('projection requires a finite bound')
                    minimum+=a*column_bound
                    continue
                if name in selected:
                    found.add(name);c=1 if a>0 else -1;negative+=int(c<0)
                    coeff.append([indices[name],Fraction(c)])
                    minimum+=max(0,a)
                else:minimum+=min(0,a)
            if found!=selected or minimum<=sign*bound:raise AssertionError('forced cover does not violate source row')
            coeff.sort(key=lambda p:p[0]);row=cut['row'];expected=len(cover)-1-negative
            actual=[[j,Fraction(a)] for j,a in row['coefficients']]
            if row['lower'] is not None or Fraction(row['upper'])!=expected or actual!=coeff:
                raise AssertionError('derived inequality does not match cover')
            metadata['checked']+=1
    except Exception as error:return dict(metadata,valid=False,error=f'{type(error).__name__}: {error}')
    return dict(metadata,valid=True,input_sha256=sha256(Path(model_path)))

if __name__=='__main__':
    try:
        result=json.loads(Path(sys.argv[2]).read_text(),parse_float=Decimal)
        check=validate_cut_proofs(sys.argv[1],result)
    except Exception as error:check=dict(valid=None,error=f'{type(error).__name__}: {error}')
    if len(sys.argv)>3:atomic_json(Path(sys.argv[3]),check)
    print(json.dumps(check,allow_nan=False))
    raise SystemExit(0 if check['valid'] is True else 1 if check['valid'] is False else 2)
