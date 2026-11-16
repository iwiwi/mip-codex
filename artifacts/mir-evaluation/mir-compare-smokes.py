import json
from pathlib import Path
records=[]
for oldlog,newlog in [('four-rounds-current-smoke8','mir-smoke8-venv'),('four-rounds-current-smoke10','mir-smoke10-venv')]:
 old=Path(Path('artifacts/'+oldlog+'.log').read_text().splitlines()[-1]);new=Path(Path('artifacts/'+newlog+'.log').read_text().splitlines()[-1]);oc=json.loads((old/'experiment.json').read_text())['cpu'];nc=json.loads((new/'experiment.json').read_text())['cpu']
 for p in sorted(new.glob('*/rust/result.json')):
  a=json.loads((old/p.relative_to(new)).read_text());b=json.loads(p.read_text());r=dict(name=p.parent.parent.name,before_cpu=oc,after_cpu=nc,before={k:a[k]for k in ['status','objective','dual_bound','nodes']},after={k:b[k]for k in ['status','objective','dual_bound','nodes']},mir_added=b['cut_stats']['mir_added'],mir_seconds=b['cut_stats']['mir_seconds'],before_result=str(old/p.relative_to(new)),after_result=str(p));records.append(r);print(r['name'],[a[k]for k in ['objective','dual_bound']],'->',[b[k]for k in ['objective','dual_bound']],r['mir_added'])
Path('artifacts/mir-smoke-comparison.json').write_text(json.dumps(dict(purpose='initial nonpaired screening; CPU differs, no speed claim',records=records),indent=2)+'\n')
