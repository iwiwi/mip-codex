import json,math
from pathlib import Path
records=[]
for cpu in [0,1]:
 run=Path(f'artifacts/mir-matrix-paired-{cpu}')
 for p in sorted(run.glob('*/before/result.json')):
  q=p.parent.parent/'after/result.json'
  if not q.exists():continue
  a=json.loads(p.read_text());b=json.loads(q.read_text());same=all((a[k]is None and b[k]is None)or(a[k]is not None and b[k]is not None and abs(a[k]-b[k])<=1e-7*(1+abs(a[k])))for k in ['objective','dual_bound'])
  records.append(dict(name=p.parent.parent.name,cpu=cpu,same_bounds=same,before={k:a[k]for k in ['status','objective','dual_bound','nodes']},after={k:b[k]for k in ['status','objective','dual_bound','nodes']},node_ratio=b['nodes']/a['nodes']if a['nodes']else None))
summary=dict(instances=len(records),same_bounds=sum(r['same_bounds']for r in records),geometric_node_ratio=math.exp(sum(math.log(r['node_ratio'])for r in records if r['node_ratio'])/sum(bool(r['node_ratio'])for r in records)),records=records)
Path('artifacts/mir-matrix-paired-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items()if k!='records'}))
