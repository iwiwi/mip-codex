from pathlib import Path
import subprocess,json,time
root=Path.cwd();out=root/'artifacts/objective-cut-repair-components';out.mkdir();seed=root/'artifacts/row-packing-bound-v2-paired-41/neos-5052403-cygnet.mps.gz/after/result.json';whole=json.load(open(seed));values=whole['values'].copy();records=[];binary=root/'target-dual-restoration/release/examples/objective_cut_repair'
for model in sorted((root/'artifacts/component-primal-cygnet-120/components').glob('component-*/model.json')):
 d=out/model.parent.name;d.mkdir();cmd=['taskset','-c','45','timeout','--signal=KILL','65',str(binary),str(model),str(seed),'5',str(d/'solution.json')];start=time.monotonic()
 with (d/'stdout.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:p=subprocess.run(cmd,stdout=stdout,stderr=stderr)
 assert p.returncode==0;solution=json.load(open(d/'solution.json'));values.update(solution['values']);r=dict(component=model.parent.name,objective=solution['objective'],wall=time.monotonic()-start);records.append(r);(out/'progress.json').write_text(json.dumps(records,indent=2))
whole.update(values=values,objective=sum(values.values()),status='feasible_component_objective_cut_diagnostic',dual_bound=None,gap=None);(out/'solution.json').write_text(json.dumps(whole));(out/'summary.json').write_text(json.dumps(dict(objective=whole['objective'],components=records),indent=2));print(whole['objective'])
