"""元MPSの整数変数・整数係数行をFractionで独立検査。許容幅なし。"""
from fractions import Fraction as F
import math,re,struct,hashlib
from pathlib import Path

def apply(names,variables,rows,result):
    proof=result.get('integer_row_proof')
    if proof is None:return variables,rows,None
    if not isinstance(proof,dict)or set(proof)!={'variables','rows'}:raise ValueError('integer bound proof schema')
    if not proof['variables'] and not proof['rows']:raise ValueError('empty integer bound proof')
    outvars=dict(variables);outrows=list(rows);sides=0
    for kind,changes,original in [('variable',proof['variables'],names),('row',proof['rows'],rows)]:
        if not isinstance(changes,list)or len(changes)>len(original):raise ValueError('integer bound count')
        last=-1
        for change in changes:
            index='source_'+kind
            if not isinstance(change,dict)or set(change)!={index,'lower_bits','upper_bits'}:raise ValueError('integer bound entry schema')
            i=change[index]
            if type(i)is not int or not last<i<len(original):raise ValueError('integer bound index/order')
            last=i
            old=variables[names[i]] if kind=='variable' else rows[i]
            if kind=='variable':
                if not old['integer']:raise ValueError('continuous variable rounding')
            elif any(a.denominator!=1 or not variables[name]['integer']for name,a in old['coefficients'].items()if a):raise ValueError('row activity is not necessarily integral')
            target=dict(old);changed=False
            for side in ['lower','upper']:
                text=change[side+'_bits']
                if text is None:continue
                if not isinstance(text,str)or re.fullmatch('[0-9a-f]{16}',text)is None:raise ValueError('integer bound binary64 schema')
                value=struct.unpack('>d',bytes.fromhex(text))[0]
                if not math.isfinite(value)or value!=math.trunc(value)or abs(value)>2**52:raise ValueError('integer bound noninteger/nonfinite value')
                bound=old[side]
                if bound is None:raise ValueError('missing original bound')
                expected=math.ceil(bound)if side=='lower'else math.floor(bound)
                if F.from_float(value)!=expected or F(expected)==bound:raise ValueError('not exact original decimal ceil/floor')
                target[side]=F(expected);changed=True;sides+=1
            if not changed:raise ValueError('empty integer bound entry')
            if kind=='variable':outvars[names[i]]=target
            else:outrows[i]=target
    return outvars,outrows,dict(valid=True,variables=len(proof['variables']),rows=len(proof['rows']),sides=sides,arithmetic='original MPS Fraction integer coefficients/types and exact ceil/floor, no tolerance',checker_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
