#!/usr/bin/env python3
"""Export completed runs with incumbents and checks for repository review."""
import json,pathlib,sys
root=pathlib.Path(sys.argv[1]);out=pathlib.Path(sys.argv[2]);meta=json.loads((root/'run.json').read_text());assert 'finished' in meta
records=[]
for p in sorted(root.glob('*.result.json')):
 r=json.loads(p.read_text());r['solution']=json.loads((root/r['solution_file']).read_text()) if (root/r['solution_file']).exists() and (root/r['solution_file']).stat().st_size else None
 check=p.with_name(p.name.replace('.result.json','.check.log'))
 if check.exists():r['check_output']=check.read_text()
 records.append(r)
assert len(records)==len(meta['instances'])*len(meta['solvers'])
out.parent.mkdir(parents=True,exist_ok=True);out.write_text(json.dumps(dict(metadata=meta,records=records),indent=2)+'\n')
for solver in meta['solvers']:
 rows=[r for r in records if r['solver']==solver];solved=sum(r['termination']=='finished' and r.get('independent_check_passed',False) and r.get('result',{}).get('status','').lower()=='optimal' for r in rows);incumbents=sum(r.get('independent_check_passed',False) for r in rows)
 print(f'{solver}: solved {solved}/{len(rows)}, checked incumbents {incumbents}/{len(rows)}')
for name in meta['instances']:
 print(name,[(r['solver'],r.get('result',{}).get('objective'),r.get('result',{}).get('dual_bound'),r['termination']) for r in records if r['instance']==name])
