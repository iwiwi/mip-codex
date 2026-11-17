"""元MPSの連続箱上で、原行乗数のLagrangian下界を有理数で独立検査する。"""
from fractions import Fraction as F
from pathlib import Path
import math,re,struct,hashlib

def bits(text):
    if not isinstance(text,str)or re.fullmatch('[0-9a-f]{16}',text)is None:raise ValueError('master bound binary64 schema')
    value=struct.unpack('>d',bytes.fromhex(text))[0]
    if not math.isfinite(value):raise ValueError('master bound nonfinite value')
    return F.from_float(value)

def verify(model_path,names,variables,rows,objective,offset,result):
    stats=result.get('row_generation')
    if stats is None:return None
    if not isinstance(stats,dict):raise ValueError('row generation stats schema')
    proof=stats.get('bound_proof')
    if proof is None:return None
    if not isinstance(proof,dict)or set(proof)!={'multipliers','normalized_bound_bits'}:raise ValueError('master bound proof schema')
    multipliers=proof['multipliers']
    if not isinstance(multipliers,list)or not multipliers or len(multipliers)>len(rows):raise ValueError('master bound multipliers schema')
    from verify_binary_graph import maximize_sense
    sign=-1 if maximize_sense(model_path)else 1
    reduced={name:sign*c for name,c in objective.items()if c};total=sign*offset;last=-1
    for pair in multipliers:
        if not isinstance(pair,list)or len(pair)!=2:raise ValueError('master bound row multiplier schema')
        i,text=pair
        if type(i)is not int or not last<i<len(rows):raise ValueError('master bound row index/order')
        last=i;y=bits(text)
        if not y:raise ValueError('master bound zero multiplier')
        row=rows[i];bound=row['lower']if y>0 else row['upper']
        if bound is None:raise ValueError('master bound missing original row side')
        total+=y*bound
        for name,a in row['coefficients'].items():reduced[name]=reduced.get(name,F(0))-y*a
    for name,c in reduced.items():
        if not c:continue
        bound=variables[name]['lower'if c>0 else'upper']
        if bound is None:raise ValueError('master bound missing original variable side')
        total+=c*bound
    claimed=bits(proof['normalized_bound_bits'])
    if claimed>total:raise ValueError('master bound exceeds exact original Lagrangian bound')
    dual=result.get('dual_bound')
    if dual is not None:
        value=float(dual)
        if not math.isfinite(value)or sign*F.from_float(value)<claimed:raise ValueError('reported dual is weaker than its master bound certificate')
    return dict(valid=True,multipliers=len(multipliers),normalized_bound=str(claimed),exact_lagrangian_bound=str(total),arithmetic='original MPS Fraction; no bound proof tolerance',checker_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
