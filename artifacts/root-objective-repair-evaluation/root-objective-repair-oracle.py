from pathlib import Path
import json,subprocess,sys,itertools,os,hashlib
sys.path.insert(0,'scripts');from validate_solution import validate
source=Path('artifacts/cover-row-packing-synthetic');out=Path('artifacts/root-objective-repair-oracle');out.mkdir();binary=Path('target-objective-repair/release/examples/objective_repair').resolve();(out/'experiment.json').write_text(json.dumps(dict(binary=str(binary),sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),source_fingerprint='10e763eef59eca62',original_absolute_tolerance='1e-6'),indent=2));records=[]
for p in sorted(source.iterdir()):
 if not p.is_dir():continue
 m=json.load(open(p/'model.json'));m['offset']=0;m['offset_enclosure']=[0,0];d=out/p.name;d.mkdir();(d/'model.json').write_text(json.dumps(m));a=json.loads(subprocess.check_output([str(binary),str(d/'model.json')],text=True));(d/'result.json').write_text(json.dumps(a));n=len(m['variables']);lines=['NAME repair','ROWS',' N obj']+[f" G {r['name']}" for r in m['rows']]+['COLUMNS']
 for j in range(n):
  lines+=[f' x{j} obj 1']
  for r in m['rows']:
   if any(k==j for k,c in r['coefficients']):lines+=[f" x{j} {r['name']} 1"]
 lines+=['RHS']+[f" rhs {r['name']} 1" for r in m['rows']]+['BOUNDS']+[f' BV b x{j}'for j in range(n)]+['ENDATA'];path=d/'model.mps';path.write_text('\n'.join(lines)+'\n');v=validate(path,a);assert v['valid']is True,v;(d/'validation.json').write_text(json.dumps(v));opt=min(sum(x)for x in itertools.product([0,1],repeat=n)if all(sum(x[j]for j,c in row['coefficients'])>=1 for row in m['rows']));assert opt<=a['objective']<=n;records.append(dict(case=p.name,objective=a['objective'],optimum=opt,valid=True));(out/'progress.json').write_text(json.dumps(dict(records=len(records),valid=len(records))))
(out/'summary.json').write_text(json.dumps(dict(cases=len(records),valid=len(records),records=records),indent=2));print(len(records))
