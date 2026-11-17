"""元MPSに対する有理実行可能点証拠をFractionで独立検査する。"""
from fractions import Fraction
from decimal import Decimal
import math,re

def verify(model_path,result):
    from verify_cover_cuts import exact_model
    proof=result.get('rational_primal_proof')
    if proof is None:return dict(valid=True,present=False)
    if type(proof)is not dict or set(proof)!={'values'}or type(proof['values'])is not list:raise AssertionError('primal witness schema')
    names,variables,rows,objective,offset=exact_model(model_path,True)
    raw=result.get('values')
    if type(raw)is not dict or set(raw)!=set(names):raise AssertionError('primal witness original variable set')
    point={}
    for name in names:
        value=raw[name]
        if type(value)not in (int,float,Decimal)or not math.isfinite(float(value)):raise AssertionError('primal witness nonfinite coordinate')
        point[name]=Fraction.from_float(float(value))
    nominal=point.copy();previous=-1
    if len(proof['values'])>4096:raise AssertionError('primal witness coordinate count')
    for coordinate in proof['values']:
        if type(coordinate)is not dict or set(coordinate)!={'variable','numerator','denominator'}:raise AssertionError('primal witness coordinate schema')
        j=coordinate['variable'];n=coordinate['numerator'];d=coordinate['denominator']
        if type(j)is not int or not previous<j<len(names):raise AssertionError('primal witness coordinate order/index')
        previous=j
        if type(n)is not str or type(d)is not str or len(n)>2500 or len(d)>2500 or re.fullmatch(r'0|-?[1-9][0-9]*',n)is None or re.fullmatch(r'[1-9][0-9]*',d)is None:raise AssertionError('primal witness rational encoding')
        n,d=int(n),int(d)
        if abs(n).bit_length()>8192 or d.bit_length()>8192 or math.gcd(n,d)!=1:raise AssertionError('primal witness rational normalization/size')
        value=Fraction(n,d);name=names[j]
        if value==nominal[name]or abs(value-Fraction(str(raw[name])))>Fraction(1,10**6):raise AssertionError('primal witness is not a distinct nearby coordinate')
        point[name]=value
    for name,v in variables.items():
        value=point[name]
        if abs(value-Fraction(str(raw[name])))>Fraction(1,10**6):raise AssertionError('primal witness serialized point distance')
        if v['integer']and value.denominator!=1:raise AssertionError('primal witness integrality')
        if v['lower']is not None and value<v['lower']or v['upper']is not None and value>v['upper']:raise AssertionError('primal witness variable bound')
    for row in rows:
        activity=sum(a*point[n]for n,a in row['coefficients'].items())
        if row['lower']is not None and activity<row['lower']or row['upper']is not None and activity>row['upper']:raise AssertionError('primal witness original row')
    exact=offset+sum(a*point[n]for n,a in objective.items());reported=result.get('objective')
    if type(reported)not in (int,float,Decimal)or not math.isfinite(float(reported)):raise AssertionError('primal witness objective missing')
    if abs(exact-Fraction(str(reported)))>Fraction(1,10**8)*(1+abs(exact)):raise AssertionError('primal witness objective consistency')
    return dict(valid=True,present=True,coordinates=len(proof['values']),exact_objective=str(exact))
