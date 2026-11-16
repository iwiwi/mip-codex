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
    domains=0
    for key,sign in [('min_domain',1),('max_domain',-1)]:
        domain=proof.get(key)
        if domain is None:continue
        if not isinstance(domain,dict) or type(domain.get('base'))!=int or type(domain.get('first_increment'))!=int:
            raise AssertionError('invalid objective domain schema')
        base=sign*offset;weights=[]
        for name,c in objective.items():
            c*=sign
            if not c:continue
            bound=variables[name]['lower' if c>0 else 'upper']
            if bound is None or bound.denominator!=1:raise AssertionError('objective domain requires integer box endpoints')
            base+=c*bound;weights.append(abs(c.numerator))
        if domain['base']!=base or domain['first_increment']!=min(weights):raise AssertionError('objective domain gap mismatch')
        domains+=1
    return dict(objective_grid_valid=True,objective_step=step,implied_integer_variables=len(implications),objective_domains=domains)


def verify_clique_exact(names,variables,rows,cut):
    proof=cut['proof'];pairs=proof['clique'];row=cut['row']
    if proof.get('aggregation',[]) or proof.get('divisor') is not None:raise AssertionError('clique cannot combine other proof kinds')
    coefficients=row['coefficients'];k=len(coefficients)
    if not 3<=k<=32 or len(pairs)!=k*(k-1)//2:raise AssertionError('clique size/pair count')
    if any(type(j)is not int or not 0<=j<len(names)or Fraction(a)not in [-1,1]for j,a in coefficients):raise AssertionError('clique coefficient')
    if [j for j,a in coefficients]!=sorted(set(j for j,a in coefficients))or proof['cover']!=[j for j,a in coefficients]:raise AssertionError('clique variable order')
    coeff={j:Fraction(a)for j,a in coefficients}
    def binary(v):return v['integer']and v['lower']is not None and v['upper']is not None and math.ceil(v['lower'])>=0 and math.floor(v['upper'])<=1
    if any(not binary(variables[names[j]])for j in coeff):raise AssertionError('clique requires binary variables')
    if row['lower']is not None or Fraction(row['upper'])!=1-sum(a<0 for a in coeff.values())or row.get('exact_integral')is not True:raise AssertionError('clique inequality')
    first=pairs[0]
    if any(proof[key]!=first[key]for key in ['source_row','source_name','upper_side']):raise AssertionError('clique first source mismatch')
    seen=set();cache={}
    for pair in pairs:
        indices=pair['variables']
        if not isinstance(indices,list)or len(indices)!=2 or any(type(j)is not int or j not in coeff for j in indices)or indices[0]>=indices[1]or tuple(indices)in seen:raise AssertionError('clique pair variables')
        seen.add(tuple(indices));ri=pair['source_row'];upper=pair['upper_side']
        if type(ri)is not int or not 0<=ri<len(rows)or type(upper)is not bool:raise AssertionError('clique source index/side')
        source=rows[ri]
        if pair['source_name']!=source['name']:raise AssertionError('clique source name')
        key=(ri,upper)
        if key not in cache:
            sign=1 if upper else-1;bound=source['upper'if upper else'lower']
            if bound is None:raise AssertionError('clique source side missing')
            minimum=Fraction(0);oriented={}
            for name,a in source['coefficients'].items():
                a*=sign
                if not a:continue
                oriented[name]=a;v=variables[name]
                if binary(v):minimum+=min(a,0)
                else:
                    b=v['lower'if a>0 else'upper']
                    if b is None:raise AssertionError('clique unbounded projection')
                    minimum+=a*b
            cache[key]=(minimum,sign*bound,oriented)
        minimum,bound,oriented=cache[key]
        for j in indices:
            a=oriented.get(names[j],Fraction(0))
            if a==0 or (a>0)!=(coeff[j]>0):raise AssertionError('clique literal does not match source sign')
            minimum+=abs(a)
        if minimum<=bound:raise AssertionError('clique pair has no strict contradiction')
    return len(pairs)


def validate_cut_proofs(model_path,result):
    cuts=result.get('cut_proofs',[])
    grid=result.get('objective_grid')
    graph=result.get('binary_graph')
    metadata=dict(valid=None,checked=0,arithmetic='Python Fraction from original MPS tokens',checker_sha256=sha256(Path(__file__)))
    if not isinstance(cuts,list):return dict(metadata,valid=False,error='cut_proofs must be a list')
    if not cuts and grid is None and graph is None:return dict(metadata,valid=True)
    try:names,variables,rows,objective,offset=exact_model(model_path,True)
    except Exception as error:return dict(metadata,error=f'cannot read exact MPS: {type(error).__name__}: {error}')
    indices={name:j for j,name in enumerate(names)}
    try:
        if graph is not None:
            from verify_binary_graph import verify
            metadata['binary_graph']=verify(model_path,result,(names,variables,rows,objective,offset))
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
            clique=proof.get('clique',[])
            if not isinstance(clique,list):raise AssertionError('bad clique pairs')
            if clique:
                count=verify_clique_exact(names,variables,rows,cut)
                metadata['checked']+=1
                metadata['clique_checked']=metadata.get('clique_checked',0)+1
                metadata['clique_pairs_checked']=metadata.get('clique_pairs_checked',0)+count
                continue
            divisor=proof.get('divisor')
            aggregation=proof.get('aggregation',[])
            if not isinstance(aggregation,list):raise AssertionError('bad aggregation terms')
            if aggregation:
                if len(aggregation)!=2 or divisor is None:raise AssertionError('aggregation requires two terms and rounding')
                first=aggregation[0]
                if any(first.get(k)!=proof[k]for k in ['source_row','source_name','upper_side']):raise AssertionError('aggregation first source mismatch')
                combined={};combined_rhs=Fraction(0);used=set()
                for term in aggregation:
                    ri=term['source_row'];weight=term['weight'];upper=term['upper_side']
                    if type(ri)is not int or not 0<=ri<len(rows)or ri in used:raise AssertionError('bad aggregate source index')
                    if type(weight)is not int or not 0<weight<=2**52 or type(upper)is not bool:raise AssertionError('bad aggregation weight/side')
                    used.add(ri);r=rows[ri]
                    if term['source_name']!=r['name']:raise AssertionError('bad aggregate source name')
                    rhs=r['upper'if upper else'lower']
                    if rhs is None or rhs.denominator!=1:raise AssertionError('aggregate source requires integer side')
                    signed=weight*(1 if upper else-1);combined_rhs+=signed*rhs
                    for name,a in r['coefficients'].items():
                        if a.denominator!=1:raise AssertionError('aggregate source requires integer coefficients')
                        combined[name]=combined.get(name,Fraction(0))+signed*a
                if abs(combined_rhs)>2**52 or any(abs(a)>2**52 for a in combined.values()):raise AssertionError('aggregate source outside exact float range')
                source=dict(coefficients=combined,upper=combined_rhs,lower=None)
                bound=combined_rhs;sign=1
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
                if aggregation:metadata['aggregate_checked']=metadata.get('aggregate_checked',0)+1
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
