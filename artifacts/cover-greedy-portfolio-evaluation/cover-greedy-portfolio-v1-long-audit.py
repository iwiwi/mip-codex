import json,sys,subprocess,signal,os,resource
from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
root=Path.cwd();spec=spec_from_file_location('warm_checker',str(root/'artifacts/cover-greedy-portfolio-v1-diagnostic/scripts/verify_cover_cuts.py'));mod=module_from_spec(spec);spec.loader.exec_module(mod);os.sched_setaffinity(0,{45});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));out=root/'artifacts/cover-greedy-portfolio-v1-long-audit';out.mkdir();manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[]
for run in sorted([root/'artifacts/cover-greedy-portfolio-v1-paired-44']):
 if not run.is_dir():continue
 assert (run/'exit.json').exists(),run
 for p in sorted(run.glob('*/*/result.json')):
  name=p.parent.parent.name;r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];v=json.load(open(p));d=out/run.name/name/p.parent.name;d.mkdir(parents=True);record=dict(source=str(p),sha256=sha256(p),cut=mod.validate_cut_proofs(model,v),point=None)
  if v.get('values')is not None:
   cmd=[str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(p),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]
   with(d/'audit.log').open('w')as stream:
    proc=subprocess.Popen(cmd,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
    try:proc.wait(timeout=600)
    except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
   record['point']={k:json.load(open(d/(k+'-validation.json')))for k in ['strict','official']}
  records.append(record);atomic_json(d/'record.json',record);atomic_json(out/'progress.json',dict(records=len(records),cut_valid=sum(r['cut']['valid']is True for r in records),points=sum(r['point']is not None for r in records),point_valid=sum(r['point']is not None and all(x['valid']is True for x in r['point'].values())for r in records)))
s=json.load(open(out/'progress.json'));atomic_json(out/'summary.json',s);atomic_json(out/'records.json',records);assert s['records']==s['cut_valid']and s['points']==s['point_valid'];atomic_json(out/'exit.json',dict(status='complete'));print(s)
