import json,sys
from pathlib import Path
sys.path.insert(0,'scripts')
from verify_binary_graph import maximize_sense
records=[]
for d in sorted(Path('artifacts/root-rounds-diagnostic10').iterdir()):
 if not(d/'2/result.json').is_file()or not(d/'4/result.json').is_file():continue
 a=json.loads((d/'2/result.json').read_text());b=json.loads((d/'4/result.json').read_text());meta=json.loads((d/'2/experiment.json').read_text());model=Path(next(x for x in meta['command']if x.endswith('.mps.gz')));sign=-1 if maximize_sense(model)else 1
 record=dict(name=d.name,before={k:a.get(k)for k in ['status','objective','dual_bound','nodes']},after={k:b.get(k)for k in ['status','objective','dual_bound','nodes']},primal='same',dual='same')
 for key,label in [('objective','primal'),('dual_bound','dual')]:
  old,new=a.get(key),b.get(key)
  if old is None or new is None:record[label]='same'if old is new else 'gained'if old is None else'lost'
  else:
   delta=sign*(new-old);tol=1e-7*(1+abs(old));record[label]='same'if abs(delta)<=tol else'improved'if(delta<0 if key=='objective'else delta>0)else'worsened'
 records.append(record)
assert len(records)==18,len(records)
Path('artifacts/four-rounds-comparison18.json').write_text(json.dumps(records,indent=2)+'\n');print(json.dumps(dict(cases=len(records),changes=[r for r in records if r['primal']!='same'or r['dual']!='same'or r['before']['status']!=r['after']['status']]),indent=2))
