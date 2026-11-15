import json,os,time,math
from pathlib import Path
import numpy as np
os.sched_setaffinity(0,{5});r=json.load(open('artifacts/fractional-cover-maximal-fast.json'));groups=r['cliques'];n=2048;flat=np.array([v for g in groups for v in g],dtype=np.int64);starts=np.array([sum(map(len,groups[:i]))for i in range(len(groups))],dtype=np.int64)
records=[]
for eta in [.03,.1,.3,1.]:
 prices=np.ones(n);coverage=np.zeros(n,dtype=np.int64);counts=np.zeros(len(groups),dtype=np.int64);best=n;best_counts=None;start=time.monotonic();step=0;last=0
 while time.monotonic()-start<2.:
  score=np.add.reduceat(prices[flat],starts);j=int(score.argmax());counts[j]+=1;coverage[groups[j]]+=1;prices[groups[j]]*=math.exp(-eta);step+=1
  if step%32==0:
   minimum=int(coverage.min())
   if minimum and step//minimum<best:best=step//minimum;best_counts=counts.copy();last=step
  if step%4096==0 and prices.max()<1e-40:prices/=prices.max()
 record=dict(eta=eta,steps=step,upper=best,last_improvement=last,seconds=time.monotonic()-start,weights=best_counts.tolist()if best_counts is not None else None);records.append(record);print({k:v for k,v in record.items()if k!='weights'},flush=True)
Path('artifacts/fractional-cover-count-diagnostic.json').write_text(json.dumps(dict(purpose='independently written greedy count cover diagnostic; no native LP weights used',cliques=groups,results=records))+'\n')
