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


def verify_mir_exact(names,variables,source,cut):
    p=cut['proof'];row=cut['row'];d=p.get('divisor');upper=p['mir']['upper_bounds']
    if type(d)is not int or not 2<=d<=2**52 or p['cover'] or p.get('clique') or p.get('aggregation'):raise AssertionError('MIR proof kinds/divisor')
    if not isinstance(upper,list)or any(type(j)is not int or not 0<=j<len(names)for j in upper)or upper!=sorted(set(upper)):raise AssertionError('MIR complements')
    sign=1 if p['upper_side'] else -1
    b=sign*source['upper' if sign==1 else 'lower'];terms=[];used=set();has_integer=False;indices={n:j for j,n in enumerate(names)}
    if b.denominator!=1:raise AssertionError('MIR integral source RHS')
    for name,a in source['coefficients'].items():
        if not a:continue
        if a.denominator!=1:raise AssertionError('MIR integral source coefficient')
        j=indices[name];v=variables[name];complement=j in upper
        if complement:used.add(j)
        bound=v['upper' if complement else 'lower']
        if bound is None or bound.denominator!=1:raise AssertionError('MIR bound')
        a*=sign;b-=a*bound;direction=-1 if complement else 1
        terms.append((j,a*direction,bound,direction,v['integer']));has_integer|=v['integer']
    if used!=set(upper)or not has_integer:raise AssertionError('MIR complement/source')
    beta=b/d;f=beta-beta.__floor__()
    if not f:raise AssertionError('MIR integral beta')
    scale=d*(1-f);rhs=scale*beta.__floor__();coeff=[]
    for j,a,bound,direction,is_integer in sorted(terms):
        alpha=a/d
        if is_integer:
            floor=alpha.__floor__();fraction=alpha-floor
            c=Fraction(floor)+max(Fraction(0),(fraction-f)/(1-f))
        else:c=min(alpha,Fraction(0))/(1-f)
        c*=scale*direction;rhs+=c*bound
        if c:coeff.append((j,c))
    actual=[(j,Fraction(a))for j,a in row['coefficients']]
    if actual!=coeff or row['lower']is not None or Fraction(row['upper'])!=rhs or row.get('exact_integral')is not True:raise AssertionError('MIR inequality mismatch')

def verify_gomory_exact(names,variables,rows,cut):
    p=cut['proof'];proof=p['gomory'];row=cut['row'];terms=proof['terms'];upper=proof['upper_bounds'];pivot=proof['pivot'];n=len(names);indices={name:j for j,name in enumerate(names)}
    if p.get('mir')is not None or p.get('clique')or p.get('aggregation')or p.get('divisor')is not None or p['cover']:raise AssertionError('Gomory mixed proof kinds')
    mixed=proof.get('mixed',False);normalized=proof.get('normalized',False)
    if type(normalized)is not bool:raise AssertionError('Gomory normalization flag')
    if type(mixed)is not bool or (not mixed and not all(v['integer']for v in variables.values()))or not 0<n<=100000:raise AssertionError('Gomory integer flags')
    if type(pivot)is not int or not 0<=pivot<n or not isinstance(terms,list)or not 0<len(terms)<=1024:raise AssertionError('Gomory shape')
    if not isinstance(upper,list)or any(type(j)is not int or not 0<=j<n for j in upper)or upper!=sorted(set(upper)):raise AssertionError('Gomory complements')
    if not variables[names[pivot]]['integer']:raise AssertionError('Gomory pivot must be integer')
    anchors=[];directions=[]
    for j,name in enumerate(names):
        d=-1 if j in upper else 1;b=variables[name]['upper'if d==-1 else'lower']
        if b is None or b.denominator!=1 or abs(b)>2**52:raise AssertionError('Gomory bound must be exact integer')
        anchors.append(int(b));directions.append(d)
    A=[0]*n;B=0;equations=[];previous=None;work=0
    for t in terms:
        i=t['row'];side=t['upper'];weight=t['weight']
        if type(i)is not int or not 0<=i<len(rows)or type(side)is not bool or type(weight)is not int or not -2**63<=weight<2**63 or weight==0:raise AssertionError('Gomory term')
        key=(i,side)
        if previous is not None and key<=previous:raise AssertionError('Gomory term order')
        previous=key;source=rows[i];sign=1 if side else-1;b=source['upper'if side else'lower'];work+=len(source['coefficients'])
        if b is None or b.denominator!=1 or abs(b)>2**52 or work>1000000:raise AssertionError('Gomory source bound/work')
        coefficients={indices[name]:int(v)*sign for name,v in source['coefficients'].items()if v}
        if any(v.denominator!=1 or abs(v)>2**52 for v in source['coefficients'].values()):raise AssertionError('Gomory source coefficients')
        for j,v in coefficients.items():A[j]+=weight*v
        B+=weight*sign*int(b);equations.append((coefficients,sign*int(b),weight,all(variables[names[j]]['integer']for j in coefficients)))
    orientation=-1 if A[pivot]<0 else 1;A=[v*orientation for v in A];B*=orientation;M=A[pivot]
    if M<=0:raise AssertionError('Gomory zero pivot')
    shifted=B-sum(v*t for v,t in zip(A,anchors));R0=shifted%M
    if R0==0:raise AssertionError('Gomory integral rhs')
    h=M-R0
    def coefficient(v,integer):
        if not mixed:return v%M
        if not integer:return v*h if v>=0 else -R0*v
        rem=v%M
        return rem*h if rem<=R0 else R0*(M-rem)
    C=[d*coefficient(d*v,variables[name]['integer'])for d,v,name in zip(directions,A,names)];rhs=(R0*h if mixed else R0)+sum(v*t for v,t in zip(C,anchors))
    for coefficients,b,w,slack_integer in equations:
        rem=coefficient(w*orientation,slack_integer);rhs-=rem*b
        for j,v in coefficients.items():C[j]-=rem*v
    common=math.gcd(rhs,*C)
    if common>1:C=[v//common for v in C];rhs//=common
    largest=max(abs(rhs),*(abs(v)for v in C));scale=1<<(largest.bit_length()-1)if normalized and largest>16384 else 1
    expected=[(j,Fraction(v,scale))for j,v in enumerate(C)if v]
    if not expected or abs(rhs)>2**52 or any(abs(v)>2**52 for v in C):raise AssertionError('Gomory exact float range')
    rhs=Fraction(rhs,scale);exact_float=lambda v:Fraction(v)if type(v)is int else Fraction.from_float(float(v))
    actual=[(j,exact_float(v))for j,v in row['coefficients']];integral=rhs.denominator==1 and all(v.denominator==1 for j,v in expected)
    if actual!=expected or row['upper']is not None or exact_float(row['lower'])!=rhs:raise AssertionError('Gomory cut mismatch')
    if (row.get('lower_enclosure')is None or [exact_float(v)for v in row['lower_enclosure']]!=[rhs,rhs])or row.get('upper_enclosure')is not None or row.get('exact_integral')is not integral or row.get('exact_coefficients')is not True or row.get('exact_coefficient_flags'):raise AssertionError('Gomory row metadata')

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


def verify_original_box(model_path,names,variables,objective,offset,result):
    proof=result.get('objective_box_proof')
    if proof is None:return None
    if not isinstance(proof,dict)or set(proof)!=set(['normalized_bound_bits']):raise ValueError('invalid objective box proof')
    text=proof['normalized_bound_bits']
    if not isinstance(text,str)or len(text)!=16:raise ValueError('invalid box bound bits')
    import struct
    bound=struct.unpack('>d',bytes.fromhex(text))[0]
    if not math.isfinite(bound):raise ValueError('nonfinite objective box bound')
    from verify_binary_graph import maximize_sense
    sign=-1 if maximize_sense(model_path)else 1
    minimum=sign*offset
    for name,c in objective.items():
        c*=sign
        if not c:continue
        v=variables[name];b=v['lower']if c>0 else v['upper']
        if b is None:raise ValueError('unbounded original objective box')
        minimum+=c*b
    if Fraction.from_float(bound)>minimum:raise ValueError('objective box bound exceeds exact original minimum')
    return dict(valid=True,normalized_bound=str(Fraction.from_float(bound)),exact_original_box_minimum=str(minimum))

def validate_cut_proofs(model_path,result):
    if result.get('rational_primal_proof')is not None:
        try:
            from verify_primal_witness import verify
            verify(model_path,result)
        except Exception as error:return dict(valid=False,error=f'primal witness: {type(error).__name__}: {error}')
    cuts=result.get('cut_proofs',[])
    grid=result.get('objective_grid')
    graph=result.get('binary_graph')
    domain_bound=result.get('objective_domain_bound')
    ray=result.get('row_ray_proof')
    packing=result.get('row_packing_proof')
    subgradient=result.get('row_subgradient_proof')
    gomory_bound=result.get('gomory_bound_proof')
    generation=result.get('row_generation')
    if generation is not None and not isinstance(generation,dict):return dict(valid=False,error='row generation stats schema')
    master_bound=(generation or{}).get('bound_proof')
    metadata=dict(valid=None,checked=0,arithmetic='Python Fraction from original MPS tokens',checker_sha256=sha256(Path(__file__)))
    if not isinstance(cuts,list):return dict(metadata,valid=False,error='cut_proofs must be a list')
    if not cuts and grid is None and graph is None and domain_bound is None and ray is None and packing is None and subgradient is None and gomory_bound is None and result.get("objective_box_proof")is None and result.get("zero_dominance_proof")is None and master_bound is None and result.get("integer_row_proof")is None:return dict(metadata,valid=True)
    try:names,variables,rows,objective,offset=exact_model(model_path,True)
    except Exception as error:return dict(metadata,error=f'cannot read exact MPS: {type(error).__name__}: {error}')
    indices={name:j for j,name in enumerate(names)}
    try:
        from verify_integer_rows import apply as apply_integer_rows
        variables,rows,metadata["integer_rows"]=apply_integer_rows(names,variables,rows,result)
        from verify_master_bound import verify as verify_master_bound
        metadata["master_bound"]=verify_master_bound(model_path,names,variables,rows,objective,offset,result)
        metadata["objective_box"]=verify_original_box(model_path,names,variables,objective,offset,result)
        from verify_zero_dominance import verify as verify_zero_dominance
        metadata["zero_dominance"]=verify_zero_dominance(names,variables,rows,objective,result)
        if graph is not None:
            from verify_binary_graph import verify
            metadata['binary_graph']=verify(model_path,result,(names,variables,rows,objective,offset))
        if grid is not None:
            metadata['objective_grid_valid']=False
            metadata.update(verify_objective_grid(names,variables,rows,objective,offset,grid))
        if domain_bound is not None:
            if grid is None:raise AssertionError('objective domain report requires its grid proof')
            from verify_binary_graph import maximize_sense
            sign=-1 if maximize_sense(model_path) else 1
            domain=grid.get('max_domain' if sign<0 else 'min_domain')
            if domain is None:raise AssertionError('objective domain side missing')
            claimed=Fraction(str(domain_bound))
            if claimed!=sign*domain['base']:raise AssertionError('objective domain report mismatch')
            dual=result.get('dual_bound')
            if dual is not None and sign*Fraction(str(dual))<sign*claimed:
                raise AssertionError('reported dual is weaker than its objective domain')
            metadata['objective_domain_report_valid']=True
        if packing is not None:
            from verify_binary_graph import maximize_sense
            sign=-1 if maximize_sense(model_path) else 1
            if not isinstance(packing,dict):raise AssertionError('packing proof schema')
            selected=packing['rows']
            if not isinstance(selected,list) or any(type(i)is not int or not 0<=i<len(rows) for i in selected) or selected!=sorted(set(selected)):raise AssertionError('packing row indices')
            if any(v['lower']!=0 or sign*objective.get(name,Fraction(0))!=1 for name,v in variables.items()):raise AssertionError('packing original cost and box')
            seen=set()
            for i in selected:
                row=rows[i]
                if row['lower']!=1 or any(a!=1 for a in row['coefficients'].values()) or not seen.isdisjoint(row['coefficients']):raise AssertionError('packing rows overlap or are not unit covers')
                seen.update(row['coefficients'])
            bound=packing['normalized_bound']
            if type(bound)is not int or bound!=sign*offset+len(selected):raise AssertionError('packing exact bound mismatch')
            dual=result.get('dual_bound')
            if dual is not None and sign*Fraction(str(dual))<bound:raise AssertionError('reported dual weaker than packing')
            metadata['row_packing_valid']=True
        if ray is not None:
            import struct
            from verify_binary_graph import maximize_sense
            sign=-1 if maximize_sense(model_path) else 1
            if not isinstance(ray,dict):raise AssertionError('ray proof schema')
            bits=ray['multiplier_bits']
            if not isinstance(bits,str) or len(bits)!=16:raise AssertionError('ray multiplier encoding')
            value=struct.unpack('>d',bytes.fromhex(bits))[0]
            if not math.isfinite(value) or value<0:raise AssertionError('ray multiplier sign')
            multiplier=Fraction.from_float(value)
            selected=ray['rows']
            if not isinstance(selected,list) or any(type(i)is not int or not 0<=i<len(rows) for i in selected) or selected!=sorted(set(selected)):raise AssertionError('ray row indices')
            if any(v['lower']!=0 or v['upper']!=1 for v in variables.values()):raise AssertionError('ray requires original unit box')
            rhs=Fraction(0);columns={name:Fraction(0) for name in names}
            for i in selected:
                row=rows[i]
                if row['lower'] is None or row['lower']<=0 or any(a<0 for a in row['coefficients'].values()):raise AssertionError('ray positive cover')
                rhs+=row['lower']
                for name,a in row['coefficients'].items():columns[name]+=a
            exact=sign*offset+multiplier*rhs+sum(min(Fraction(0),sign*objective.get(name,Fraction(0))-multiplier*columns[name]) for name in names)
            bound=Fraction(str(ray['normalized_bound']))
            if bound>exact:raise AssertionError('ray bound exceeds exact Lagrangian')
            dual=result.get('dual_bound')
            if dual is not None and sign*Fraction(str(dual))<bound:raise AssertionError('reported dual weaker than ray')
            metadata['row_ray_valid']=True
        if subgradient is not None:
            import struct
            from verify_binary_graph import maximize_sense
            sign=-1 if maximize_sense(model_path) else 1
            if not isinstance(subgradient,dict):raise AssertionError('subgradient proof schema')
            entries=subgradient['multipliers']
            if not isinstance(entries,list):raise AssertionError('subgradient entries')
            if any(v['lower']!=0 or v['upper']!=1 for v in variables.values()):raise AssertionError('subgradient requires original unit box')
            columns={name:Fraction(0)for name in names};rhs=Fraction(0);previous=-1
            for entry in entries:
                if not isinstance(entry,list)or len(entry)!=2:raise AssertionError('subgradient entry schema')
                i,bits=entry
                if type(i)is not int or not previous<i<len(rows):raise AssertionError('subgradient row order/index')
                previous=i
                if not isinstance(bits,str)or len(bits)!=16:raise AssertionError('subgradient multiplier encoding')
                value=struct.unpack('>d',bytes.fromhex(bits))[0]
                if not math.isfinite(value)or value<=0:raise AssertionError('subgradient multiplier sign')
                y=Fraction.from_float(value);row=rows[i]
                if row['lower']is None or row['lower']<=0 or any(a<0 for a in row['coefficients'].values()):raise AssertionError('subgradient positive cover')
                rhs+=y*row['lower']
                for name,a in row['coefficients'].items():columns[name]+=y*a
            exact=sign*offset+rhs+sum(min(Fraction(0),sign*objective.get(name,Fraction(0))-columns[name])for name in names)
            bound=Fraction(str(subgradient['normalized_bound']))
            if bound>exact:raise AssertionError('subgradient bound exceeds exact Lagrangian')
            dual=result.get('dual_bound')
            if dual is not None and sign*Fraction(str(dual))<bound:raise AssertionError('reported dual weaker than subgradient')
            metadata['row_subgradient_valid']=True
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
            if proof.get('gomory') is not None:
                verify_gomory_exact(names,variables,rows,cut)
                metadata['checked']+=1
                metadata['gomory_checked']=metadata.get('gomory_checked',0)+1
                continue
            if proof.get('mir') is not None:
                verify_mir_exact(names,variables,source,cut)
                metadata['checked']+=1
                metadata['mir_checked']=metadata.get('mir_checked',0)+1
                continue
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
                if len(aggregation)not in (2,3) or divisor is None:raise AssertionError('aggregation requires two or three terms and rounding')
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
        if gomory_bound is not None:
            import struct
            all_rows=list(rows)
            for cut in cuts:
                row=cut['row'];number=lambda v:Fraction(v)if type(v)is int else Fraction.from_float(float(v))
                all_rows.append(dict(coefficients={names[j]:number(a)for j,a in row['coefficients']},lower=None if row['lower']is None else number(row['lower']),upper=None if row['upper']is None else number(row['upper'])))
            box_lower={name:v['lower']for name,v in variables.items()};box_upper={name:v['upper']for name,v in variables.items()};steps=gomory_bound.get('bounds',[]);box_work=0
            if not isinstance(steps,list)or len(steps)>1024:raise AssertionError('Gomory box proof length')
            for step in steps:
                i=step['source_row'];j=step['variable'];upper_side=step['upper_side'];lower_side=step['lower'];bits=step['value_bits']
                if type(i)is not int or not 0<=i<len(all_rows)or type(j)is not int or not 0<=j<len(names)or type(upper_side)is not bool or type(lower_side)is not bool or not isinstance(bits,str)or len(bits)!=16:raise AssertionError('Gomory box proof metadata')
                row=all_rows[i];box_work+=len(row['coefficients'])
                if box_work>2000000:raise AssertionError('Gomory box proof work')
                sign_row=1 if upper_side else-1;b=row['upper'if upper_side else'lower'];a=sign_row*row['coefficients'].get(names[j],Fraction(0))
                if b is None or a==0 or (a<0)!=lower_side:raise AssertionError('Gomory box row side/direction')
                others=Fraction(0)
                for name,v in row['coefficients'].items():
                    if name==names[j]or v==0:continue
                    coefficient=sign_row*v;bound=box_lower[name]if coefficient>0 else box_upper[name]
                    if bound is None:raise AssertionError('Gomory box missing dependency')
                    others+=coefficient*bound
                exact_bound=(sign_row*b-others)/a;value=struct.unpack('>d',bytes.fromhex(bits))[0]
                if not math.isfinite(value):raise AssertionError('Gomory box nonfinite bound')
                value=Fraction.from_float(value)
                if lower_side:
                    if value>exact_bound:raise AssertionError('Gomory box lower bound too strong')
                    old=box_lower[names[j]];box_lower[names[j]]=value if old is None else max(old,value)
                else:
                    if value<exact_bound:raise AssertionError('Gomory box upper bound too strong')
                    old=box_upper[names[j]];box_upper[names[j]]=value if old is None else min(old,value)
            metadata['gomory_box_steps']=len(steps)
            terms=gomory_bound['multipliers'];previous=-1;weights={}
            for i,bits in terms:
                if type(i)is not int or not previous<i<len(all_rows)or not isinstance(bits,str)or len(bits)!=16:raise AssertionError('Gomory bound multiplier order/encoding')
                previous=i;value=struct.unpack('>d',bytes.fromhex(bits))[0]
                if not math.isfinite(value)or value==0:raise AssertionError('Gomory bound multiplier value')
                weights[i]=Fraction.from_float(value)
            sign=1
            # MPS OBJSENSE is authoritative; the result need not expose it.
            import gzip
            opener=gzip.open if str(model_path).endswith('.gz')else open
            with opener(model_path,'rt')as stream:
                lines=iter(stream)
                for line in lines:
                    t=line.split()
                    if t and t[0]=='OBJSENSE':
                        if len(t)>1:sense=t[1]
                        else:
                            sense=None
                            for candidate in lines:
                                values=candidate.split()
                                if values and not values[0].startswith('*'):sense=values[0];break
                            if sense is None:raise AssertionError('missing OBJSENSE')
                        if sense.upper()not in ['MIN','MINIMIZE','MAX','MAXIMIZE']:raise AssertionError('invalid OBJSENSE')
                        sign=-1 if sense.upper().startswith('MAX')else 1;break
            rhs=sign*offset;columns={name:Fraction(0)for name in names}
            for i,w in weights.items():
                row=all_rows[i];b=row['lower'if w>0 else'upper']
                if b is None:raise AssertionError('Gomory bound row side')
                rhs+=w*b
                for name,a in row['coefficients'].items():columns[name]+=w*a
            exact=rhs
            for name in names:
                reduced=sign*objective.get(name,Fraction(0))-columns[name]
                if reduced:
                    bound=box_lower[name]if reduced>0 else box_upper[name]
                    if bound is None:raise AssertionError('Gomory bound missing variable bound')
                    exact+=reduced*bound
            declared=Fraction(str(gomory_bound['normalized_bound']))
            if declared>exact:raise AssertionError('Gomory bound exceeds exact Lagrangian')
            dual=result.get('dual_bound')
            if dual is not None and sign*Fraction(str(dual))<declared:raise AssertionError('reported bound weaker than Gomory proof')
            metadata['gomory_bound_valid']=True
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
