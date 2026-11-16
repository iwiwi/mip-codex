import json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_binary_graph import maximize_sense
records=[]
for label in ['smoke8','smoke10']:
 before_run=Path(Path(f'artifacts/four-rounds-current-{label}.log').read_text().strip().splitlines()[-1]);after_run=Path(Path(f'artifacts/static-pricing-final-{label}.log').read_text().strip().splitlines()[-1])
 for p in after_run.glob('*/rust/result.json'):
  name=p.parent.parent.name.removesuffix('.mps.gz');a=json.loads((before_run/p.relative_to(after_run)).read_text());b=json.loads(p.read_text());meta=json.loads((after_run/'experiment.json').read_text());model=Path('data/raw/benchmark')/(name+'.mps.gz');sign=-1 if maximize_sense(model)else 1
  record=dict(name=name,before={k:a.get(k)for k in ['status','objective','dual_bound','nodes']},after={k:b.get(k)for k in ['status','objective','dual_bound','nodes']},primal='same',dual='same')
  for key,label2 in [('objective','primal'),('dual_bound','dual')]:
   old,new=a.get(key),b.get(key)
   if old is None or new is None:record[label2]='same'if old is new else'gained'if old is None else'lost'
   else:
    delta=sign*(new-old);tol=1e-7*(1+abs(old));record[label2]='same'if abs(delta)<=tol else'improved'if(delta<0 if key=='objective'else delta>0)else'worsened'
  records.append(record)
assert len(records)==18,len(records)
Path('artifacts/static-pricing-final-comparison18.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(dict(cases=len(records),changes=[r for r in records if r['primal']!='same'or r['dual']!='same'or r['before']['status']!=r['after']['status']]),indent=2))
