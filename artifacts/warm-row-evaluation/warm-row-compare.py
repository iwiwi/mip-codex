import json
from pathlib import Path
all_records=[]
for label in ['smoke8','smoke10']:
 before=Path(Path(f'artifacts/zero-restoration-specialized-{label}.log').read_text().strip().splitlines()[-1])
 after=Path(Path(f'artifacts/warm-row-{label}.log').read_text().strip().splitlines()[-1])
 assert (after/'experiment.json').is_file()
 for p in sorted(after.glob('*/rust/result.json')):
  b=json.loads((before/p.relative_to(after)).read_text());a=json.loads(p.read_text());keys=['status','objective','dual_bound']
  r=dict(name=p.parent.parent.name,before={k:b.get(k)for k in keys},after={k:a.get(k)for k in keys},same=all(a.get(k)==b.get(k)for k in keys),before_nodes=b['nodes'],after_nodes=a['nodes'],extensions=a.get('lp_warm_row_extensions'))
  all_records.append(r)
Path('artifacts/warm-row-comparison18.json').write_text(json.dumps(all_records,indent=2)+'\n')
print(json.dumps(dict(cases=len(all_records),same=sum(r['same']for r in all_records),changes=[r for r in all_records if not r['same']],extensions=sum(r['extensions']or 0 for r in all_records)),indent=2))
