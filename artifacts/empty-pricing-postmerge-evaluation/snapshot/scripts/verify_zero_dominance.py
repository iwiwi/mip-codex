"""Verify original-model zero-cost singleton dominance with exact MPS Fractions."""
from fractions import Fraction
import math,struct,hashlib
from pathlib import Path

def verify(names,variables,rows,objective,result):
    proof=result.get('zero_dominance_proof')
    if proof is None:return None
    if not isinstance(proof,dict)or set(proof)!={'fixes'}:raise ValueError('zero dominance proof schema')
    fixes=proof['fixes']
    if not isinstance(fixes,list)or not fixes or len(fixes)>len(names):raise ValueError('zero dominance fixes schema')
    incidence={}
    for i,row in enumerate(rows):
        for name,a in row['coefficients'].items():
            if a:incidence.setdefault(name,[]).append((i,a))
    last=-1
    for f in fixes:
        if not isinstance(f,dict)or set(f)!={'variable','source_row','upper_endpoint','value_bits'}:raise ValueError('zero dominance fix schema')
        j,i=f['variable'],f['source_row']
        if type(j)is not int or not last<j<len(names)or type(i)is not int or not 0<=i<len(rows)or type(f['upper_endpoint'])is not bool:raise ValueError('zero dominance indices/order')
        last=j;name=names[j]
        if objective.get(name,Fraction(0))!=0:raise ValueError('nonzero original objective')
        terms=incidence.get(name,[])
        if len(terms)!=1 or terms[0][0]!=i:raise ValueError('not an original singleton')
        a=terms[0][1];row=rows[i]
        if row['lower']is None and row['upper']is not None:upper=a<0
        elif row['upper']is None and row['lower']is not None:upper=a>0
        else:raise ValueError('row is not one sided')
        if upper!=f['upper_endpoint']:raise ValueError('endpoint does not relax original row')
        v=variables[name];bound=v['upper'if upper else'lower']
        if bound is None:raise ValueError('missing finite endpoint')
        expected=Fraction(math.floor(bound)if upper else math.ceil(bound))if v['integer']else bound
        text=f['value_bits']
        if not isinstance(text,str)or len(text)!=16:raise ValueError('value bits schema')
        value=struct.unpack('>d',bytes.fromhex(text))[0]
        if not math.isfinite(value)or Fraction.from_float(value)!=expected:raise ValueError('not the exact original endpoint')
        if v['lower']is not None and expected<v['lower']or v['upper']is not None and expected>v['upper']:raise ValueError('endpoint outside original bounds')
    return dict(valid=True,fixed=len(fixes),arithmetic='original MPS Fraction singleton/sign/objective/bounds',checker_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
