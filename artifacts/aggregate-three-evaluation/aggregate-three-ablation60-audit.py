import json,sys,subprocess,signal,os
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from importlib.util import spec_from_file_location,module_from_spec
spec=spec_from_file_location('three_checker','/home/ubuntu/mip-codex/artifacts/aggregate-three-grid-diagnostic-20261116/scripts/verify_cover_cuts.py');checker_module=module_from_spec(spec);spec.loader.exec_module(checker_module);validate_cut_proofs=checker_module.validate_cut_proofs
os.sched_setaffinity(0,{37});root=Path.cwd();out=root/'artifacts/aggregate-three-ablation60-audit';out.mkdir(exist_ok=False);manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};records=[]
for run in sorted(root.glob('artifacts/aggregate-three-ablation60-paired-*')):
 if not run.is_dir():continue
 for p in sorted(run.glob('*/*/result.json')):
  name=p.parent.parent.name;r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];v=json.loads(p.read_text());d=out/run.name/name/p.parent.name;d.mkdir(parents=True);record=dict(source=str(p),sha256=sha256(p),cut=validate_cut_proofs(model,v),point=None)
  if v.get('values')is not None:
   cmd=[sys.executable,str(root/'scripts/benchmark_validate.py'),str(model),str(p),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]
   with (d/'audit.log').open('w')as stream:
    proc=subprocess.Popen(cmd,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
    try:proc.wait(timeout=600)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
   record['point']={kind:json.loads((d/(kind+'-validation.json')).read_text())for kind in ['strict','official']}
  records.append(record);atomic_json(d/'record.json',record)
summary=dict(records=len(records),cut_valid=sum(r['cut']['valid']is True for r in records),points=sum(r['point']is not None for r in records),point_valid=sum(r['point']is not None and all(v['valid']is True for v in r['point'].values())for r in records));atomic_json(out/'summary.json',summary);atomic_json(out/'records.json',records);print(json.dumps(summary));assert summary['cut_valid']==summary['records']and summary['points']==summary['point_valid']
