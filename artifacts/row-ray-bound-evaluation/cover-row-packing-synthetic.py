from pathlib import Path
import json,subprocess,random,itertools
r=random.Random(202611161547);out=Path('artifacts/cover-row-packing-synthetic');out.mkdir();records=[]
for k in range(300):
 n=r.randrange(2,11);offset=r.randrange(-5,6);rows=[]
 for i in range(r.randrange(1,16)):
  a=[(j,1) for j in range(n) if r.random()<.4]
  if not a:a=[(r.randrange(n),1)]
  rows.append(dict(name=f'r{i}',lower=1,upper=None,coefficients=a,exact_integral=True,lower_enclosure=[1,1]))
 m=dict(name=str(k),maximize=False,offset=offset,offset_enclosure=[offset,offset],variables=[dict(name=f'x{j}',lower=0,upper=1,integer=True,objective=1,objective_enclosure=[1,1],lower_enclosure=[0,0],upper_enclosure=[1,1]) for j in range(n)],rows=rows)
 p=out/f'{k:03}';p.mkdir();(p/'model.json').write_text(json.dumps(m));a=json.loads(subprocess.check_output(['target-dual-restoration/release/examples/cover_row_packing',str(p/'model.json')],text=True));(p/'result.json').write_text(json.dumps(a));seen=set()
 for i in a['selected_rows']:
  cols={j for j,v in rows[i]['coefficients']};assert cols.isdisjoint(seen);seen|=cols
 opt=min(offset+sum(x) for x in itertools.product([0,1],repeat=n) if all(sum(x[j] for j,v in row['coefficients'])>=1 for row in rows));assert a['bound']<=opt;records.append(dict(case=k,valid=True,optimum=opt,bound=a['bound'],exchanges=a['exchanges']))
(out/'summary.json').write_text(json.dumps(dict(cases=len(records),valid=len(records),exchanges=sum(a['exchanges'] for a in records),records=records),indent=2));print(len(records),'valid')
