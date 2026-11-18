"""Strict original-MPS integer spectral certificate; independent direct Walsh sums."""
from fractions import Fraction
import itertools,json,sys
from pathlib import Path
from verify_cover_cuts import exact_model
from verify_binary_graph import maximize_sense

def packing_graph(path,exact=None):
 names,variables,rows,cost,offset=exact if exact is not None else exact_model(path,True);n=len(names);index={v:i for i,v in enumerate(names)}
 assert 0<n<=4096 and n&(n-1)==0
 coefficients=[cost.get(v,Fraction(0))for v in names];assert all(c==coefficients[0]and c.denominator==1 and abs(c)<=2**52 for c in coefficients)
 assert coefficients[0]>0 if maximize_sense(path)else coefficients[0]<0
 assert all(v['integer']and v['lower']==0 and v['upper']==1 for v in variables.values())
 conflict=[0]*n
 for row in rows:
  lower,upper=row['lower'],row['upper'];entries=[(index[v],a)for v,a in row['coefficients'].items()if a]
  if lower is None and upper is None:continue
  if entries and all(a<0 for _,a in entries):lower,upper=(-upper if upper is not None else None),(-lower if lower is not None else None);entries=[(v,-a)for v,a in entries]
  assert lower is None or lower<=0
  assert all(a>0 and a.denominator==1 and a<=2**52 for _,a in entries)
  if upper is None:continue
  values=sorted(a for _,a in entries);assert upper>=max(values,default=0)
  if len(entries)<2:continue
  assert upper<values[0]+values[1]
  for a,b in itertools.combinations([v for v,_ in entries],2):conflict[a]|=1<<b;conflict[b]|=1<<a
 return names,conflict

def verify(path,certificate,graph=None):
 names,conflict=graph if graph is not None else packing_graph(path);n=len(names);c=certificate
 assert c['recognized']is True and type(c['vertices'])is int and c['vertices']==n
 labels=c['vertex_labels'];weights=c['weights'];assert len(labels)==n and all(type(v)is int for v in labels)and sorted(labels)==list(range(n))
 assert len(weights)==n and all(type(w)is int for w in weights)and weights[0]==0
 for a in range(n):
  for b in range(a+1,n):
   if not conflict[a]&(1<<b):assert weights[labels[a]^labels[b]]<=0
 eigenvalues=[sum(w*(-1 if(s&d).bit_count()%2 else 1)for d,w in enumerate(weights))for s in range(n)]
 degree=sum(weights);minimum=min(eigenvalues);assert type(c['degree'])is int and c['degree']==degree
 assert type(c['minimum_eigenvalue'])is int and c['minimum_eigenvalue']==minimum
 assert minimum<0 and degree>minimum
 upper=n*(-minimum)//(degree-minimum);assert type(c['upper'])is int and c['upper']==upper
 return dict(valid=True,vertices=n,degree=degree,minimum_eigenvalue=minimum,upper=upper,method='original MPS Fraction domain/packing; nonedge signs; direct integer Walsh sums; no LP status trusted')

if __name__=='__main__':
 path,source=sys.argv[1:3];print(json.dumps(verify(Path(path),json.load(open(source)))))
