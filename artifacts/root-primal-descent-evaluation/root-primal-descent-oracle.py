from pathlib import Path
import json,subprocess,sys,itertools,shutil
sys.path.insert(0,'scripts');from validate_solution import validate
from benchmark_common import require_solver_origin,sha256
root=Path.cwd();repo=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/root-primal-descent-oracle';out.mkdir();(out/'bin').mkdir();binary=repo/'target/release/examples/primal_descent';info=require_solver_origin(repo,binary);shutil.copy2(binary,out/'bin/primal_descent');(out/'experiment.json').write_text(json.dumps(dict(build_info=info,binary_sha256=sha256(binary)),indent=2));records=[]
for p in sorted((root/'artifacts/root-objective-repair-oracle').iterdir()):
 if not p.is_dir() or not(p/'model.mps').exists():continue
 m=json.load(open(p/'model.json'));n=len(m['variables']);d=out/p.name;d.mkdir();model=p/'model.mps';seed=dict(objective=n,values={v['name']:1 for v in m['variables']});(d/'seed.json').write_text(json.dumps(seed));cmd=[str(out/'bin/primal_descent'),str(model),str(d/'seed.json'),'1',str(d/'result.json')]
 a=json.loads(subprocess.check_output(cmd,text=True));assert a['source_fingerprint']==info['source_fingerprint'];r=json.load(open(d/'result.json'));check=validate(model,r);assert check['valid']is True;(d/'validation.json').write_text(json.dumps(check));opt=min(sum(x)for x in itertools.product([0,1],repeat=n)if all(sum(x[j]for j,c in row['coefficients'])>=1 for row in m['rows']));assert opt<=r['objective']<=n;records.append(dict(case=p.name,objective=r['objective'],optimum=opt,valid=True));(out/'progress.json').write_text(json.dumps(dict(records=len(records),valid=len(records))))
(out/'summary.json').write_text(json.dumps(dict(cases=len(records),valid=len(records),records=records),indent=2));print(len(records))
