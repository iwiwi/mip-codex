import json
from pathlib import Path
out=[]
for folder in ['graph-root-pairs','graph-root-tie-pairs','graph-root-repeat','graph-root-long-pairs','graph-root-long-native']:
 for p in Path('artifacts',folder).rglob('result.json'):
  try:r=json.loads(p.read_text())
  except json.JSONDecodeError:continue
  g=r.get('binary_graph') or {};checks={}
  for name in ['strict-validation','official-validation','proof']:
   q=p.parent/(name+'.json')
   if q.exists():checks[name]=json.loads(q.read_text()).get('valid')
  out.append(dict(path=str(p),status=r['status'],objective=r.get('objective'),dual_bound=r.get('dual_bound'),nodes=r.get('nodes'),seconds=r.get('seconds'),graph={k:g.get(k)for k in ['root_initial_upper','root_upper','root_variant_uppers','root_color_seconds','remaining_upper','best']},checks=checks))
Path('artifacts/graph-root-summary.json').write_text(json.dumps(out,indent=2)+'\n')
for r in out:print(r['path'],r['objective'],r['dual_bound'],r['checks'])
