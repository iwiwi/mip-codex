#!/usr/bin/env python3
"""Independent original-MPS Fraction reconstruction and streaming GF(2) proof."""
from fractions import Fraction as F
from verify_cover_cuts import exact_model

def validate_parity_proof(path, result):
    proof=result.get('parity_proof')
    if proof is None:return dict(valid=True,applicable=False)
    try:
        names,variables,rows,cost,offset=exact_model(path,True)
        binary=[j for j,n in enumerate(names)if variables[n]['integer']and variables[n]['lower']==0 and variables[n]['upper']==1]
        assert binary and proof['binary_variables']==binary
        lookup={names[j]:k for k,j in enumerate(binary)};degree={n:0 for n in names}
        assert all(v['integer']for v in variables.values())
        for row in rows:
            for n in row['coefficients']:degree[n]+=1
        equations={};aux=[]
        for row in rows:
            rhs=row['upper'];assert rhs is not None and row['lower']==rhs and rhs.denominator==1
            mask=0;private=[]
            for n,a in row['coefficients'].items():
                assert a.denominator==1
                if n in lookup:mask^=(int(a)%2)<<lookup[n]
                else:assert abs(a)==2 and degree[n]==1;private.append(names.index(n))
            assert len(private)==1;aux+=private;value=int(rhs)%2
            while mask:
                pivot=mask.bit_length()-1
                if pivot not in equations:equations[pivot]=(mask,value);break
                old,b=equations[pivot];mask^=old;value^=b
            else:assert value==0
        assert aux==proof['auxiliary_variables'] and proof['row_count']==len(rows)
        assert len(equations)==len(binary)==proof['rank']
        assert proof['pivots']==list(range(len(binary)))
        assert all(n in lookup or degree[n]==1 for n in names)
        bits=0
        for pivot,(mask,value)in sorted(equations.items()):
            value^=(mask&bits).bit_count()%2
            bits|=value<<pivot
        point={n:F((bits>>k)&1)for n,k in lookup.items()}
        for i,row in enumerate(rows):
            name=names[aux[i]];remainder=row['upper']-sum((a*point[n]for n,a in row['coefficients'].items()if n!=name),F(0));point[name]=remainder/row['coefficients'][name]
        assert all(x.denominator==1 for x in point.values())
        for n,v in variables.items():
            if v['lower']is not None:assert point[n]>=v['lower']
            if v['upper']is not None:assert point[n]<=v['upper']
        assert set(result['values'])==set(names)
        assert all(F(result['values'][n])==point[n]for n in names)
        objective=offset+sum((c*point[n]for n,c in cost.items()),F(0));bound=F(result['dual_bound'])
        import highspy
        from validate_solution import original_lp
        maximize=original_lp(path).sense_==highspy.ObjSense.kMaximize
        assert bound>=objective if maximize else bound<=objective
        assert abs(F(result['objective'])-objective)<=F(1,10**8)*(1+abs(objective))
        return dict(valid=True,applicable=True,rank=len(equations),binary=len(binary),rows=len(rows),exact_objective=str(objective),criterion='unique integer solution by streaming highest-bit elimination and exact original-row reconstruction')
    except Exception as error:
        return dict(valid=False,applicable=True,error=f'{type(error).__name__}: {error}')
