#!/usr/bin/env python3
"""元MPSのFractionから構造認識と根彩色を独立検査。探索木の全再生はしない。"""
import gzip,itertools
from decimal import Decimal
from fractions import Fraction
from pathlib import Path
from benchmark_common import sha256

def maximize_sense(path):
 opener=gzip.open if str(path).endswith('.gz')else open
 section='';maximize=False
 with opener(path,'rt')as stream:
  for line in stream:
   if not line.strip()or line.startswith('*'):continue
   t=line.split()
   if not line[0].isspace():
    section=t[0]
    if section=='ROWS':break
    if section!='OBJSENSE':continue
    t=t[1:]
    if not t:continue
   elif section!='OBJSENSE':continue
   if len(t)!=1 or t[0]not in ['MAX','MIN']:raise AssertionError('unsupported objective sense')
   maximize=t[0]=='MAX'
 return maximize

def verify(model_path,result,exact):
 names,variables,rows,cost,offset=exact;stats=result['binary_graph'];n=len(names)
 assert result['binary_graph_enabled']is True and result['lp_engine']=='BinaryGraph'
 assert stats['vertices']==n and 0<n<=4096
 assert sorted(stats['order'])==list(range(n))
 maximize=maximize_sense(model_path);coeff=[cost.get(name,Fraction(0))for name in names]
 assert all(c==coeff[0]and c.denominator==1 and abs(c)<=2**52 for c in coeff)
 assert coeff[0]>0 if maximize else coeff[0]<0
 assert all(v['integer']and v['lower']==0 and v['upper']==1 for v in variables.values())
 index={name:j for j,name in enumerate(names)};edges=set()
 for row in rows:
  lower=row['lower'];upper=row['upper']
  if lower is None and upper is None:continue
  entries=[(index[name],a)for name,a in row['coefficients'].items()if a]
  if entries and all(a<0 for _,a in entries):
   lower,upper=(-upper if upper is not None else None),(-lower if lower is not None else None)
   entries=[(j,-a)for j,a in entries]
  assert lower is None or lower<=0
  assert all(a>0 and a.denominator==1 and a<=2**52 for _,a in entries)
  if upper is None:continue
  weights=sorted(a for _,a in entries)
  assert upper>=max(weights,default=0)
  if len(entries)<2:continue
  assert upper<weights[0]+weights[1]
  for a,b in itertools.combinations(sorted(j for j,_ in entries),2):edges.add((a,b))
 assert stats['conflict_edges']==len(edges)
 colors=stats['root_colors'];root_upper=stats['root_upper']
 coloring_upper=stats.get('root_coloring_upper')
 if coloring_upper is None:coloring_upper=root_upper
 assert type(coloring_upper)is int and root_upper<=coloring_upper<=n
 assert type(root_upper)is int and 0<=root_upper<=n
 if colors:
  assert len(colors)==n and all(type(c)is int and 1<=c<=coloring_upper for c in colors)
  assert max(colors)==coloring_upper
  groups={}
  for j,c in enumerate(colors):groups.setdefault(c,[]).append(j)
  for group in groups.values():
   assert all((a,b)in edges for a,b in itertools.combinations(group,2))
 else:assert root_upper==n
 fractional=stats.get('fractional_cover')
 certificate=fractional.get('certificate')if fractional else None
 coverage=[0]*n;weighted_groups=[]
 if certificate:
  cliques=certificate['cliques'];counts=certificate['counts']
  assert len(cliques)==len(counts)>0
  for group,count in zip(cliques,counts):
   assert type(count)is int and 0<count<=2**32-1
   assert group and all(type(j)is int and 0<=j<n for j in group)and len(set(group))==len(group)
   assert all(tuple(sorted((a,b)))in edges for a,b in itertools.combinations(group,2))
   for j in group:coverage[j]+=count
   weighted_groups.append((set(group),count))
  total=sum(counts);minimum=min(coverage)
  assert minimum>0 and certificate['total']==total and certificate['minimum']==minimum
  assert type(certificate['upper'])is int and certificate['upper']==total//minimum
  assert root_upper==min(coloring_upper,certificate['upper'])
 else:assert root_upper==coloring_upper
 def fractional_prefix(prefix):
  if not prefix:return 0
  chosen=set(prefix)
  return sum(w for group,w in weighted_groups if group&chosen)//min(coverage[j]for j in prefix)
 best=stats['best'];upper=stats['upper'];complete=stats['complete']
 assert type(best)is int and type(upper)is int and 0<=best<=upper<=root_upper and type(complete)is bool
 remaining=stats.get('remaining_upper',root_upper)
 completed=stats.get('root_branches_completed',0)
 assert type(completed)is int and 0<=completed<=n
 assert type(remaining)is int and 0<=remaining<=root_upper
 if colors:
  candidates=stats.get('root_prefix_colorings',[])
  if candidates:
   root_order=stats['root_search_order']
   assert len(root_order)==n and all(type(j)is int for j in root_order)and set(root_order)==set(range(n))
   for coloring in candidates:
    assert len(coloring)==n and all(type(c)is int and 1<=c<=n for c in coloring)
    groups={}
    for j,c in enumerate(coloring):groups.setdefault(c,[]).append(j)
    for group in groups.values():
     assert all((a,b)in edges for a,b in itertools.combinations(group,2))
   assert coloring_upper==min(len(set(c))for c in candidates)
   prefix=root_order[:n-completed]
   prefix_bound=min(len({c[j]for j in prefix})for c in candidates)
   if certificate:prefix_bound=min(prefix_bound,fractional_prefix(prefix))
   assert remaining>=prefix_bound
  else:
   rank={j:k for k,j in enumerate(stats['order'])}
   root_order=sorted(range(n),key=lambda j:(colors[j],rank[j]))
   prefix=root_order[:n-completed]
   assert remaining>=max((colors[j]for j in prefix),default=0)
 else:assert completed==0 and remaining==n
 assert upper==(best if complete else max(best,remaining))
 point=result['values'];chosen=[j for j,name in enumerate(names)if Fraction(point[name])==1]
 assert set(point)==set(names)and len(chosen)==best and all(Fraction(v)in [0,1]for v in point.values())
 assert all(not(a in chosen and b in chosen)for a,b in edges)
 if complete:assert result['status']=='optimal'
 else:assert result['status']in ['node_limit','time_limit']
 assert result['lp_solves']==0
 exact_bound=offset+coeff[0]*upper;reported=Fraction(result['dual_bound'])
 assert reported>=exact_bound if maximize else reported<=exact_bound
 return dict(valid=True,checker_sha256=sha256(Path(__file__)),vertices=n,conflict_edges=len(edges),root_coloring_valid=True,remaining_coloring_valid=True,completed_root_branches=completed,
             root_upper=root_upper,upper=upper,best=best,
             fractional_cover_valid=bool(certificate),
             final_bound_source='root coloring/fractional cover/trivial cardinality'if upper==root_upper else'internal completed branches and remaining-prefix coloring; not independently replayed',
             independent_optimality=best==root_upper)
