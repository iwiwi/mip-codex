"""Exact original-MPS row dominance; never trusts rounded solver coefficients."""
from fractions import Fraction as F
from verify_cover_cuts import exact_model

def validate_row_dominance(path,result):
 proofs=result.get('row_dominance_proofs',[])
 if not proofs:return dict(valid=True,checked=0)
 try:
  _,_,rows,_,_=exact_model(path,True);removed={p['removed']for p in proofs};assert len(removed)==len(proofs)
  for p in proofs:
   i,k=p['removed'],p['keeper'];assert type(i)is int and type(k)is int and 0<=k<i<len(rows) and k not in removed
   row,keeper=rows[i],rows[k];assert row['name']==p['removed_name']and keeper['name']==p['keeper_name'];assert row['coefficients']==keeper['coefficients']and row['coefficients']
   for source in [row,keeper]:
    for a in [*source['coefficients'].values(),source['lower'],source['upper']]:
     if a is not None:assert a.denominator==1 and abs(a)<=2**52
   if row['lower']is not None:assert keeper['lower']is not None and keeper['lower']>=row['lower']
   if row['upper']is not None:assert keeper['upper']is not None and keeper['upper']<=row['upper']
  return dict(valid=True,checked=len(proofs),criterion='exact original coefficient equality and retained inner row range')
 except Exception as error:return dict(valid=False,checked=0,error=f'{type(error).__name__}: {error}')
