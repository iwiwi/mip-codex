"""Incremental original-model audit of completed, immutable screen outputs."""
import json,os,sys,subprocess,resource,signal,time
from pathlib import Path
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
from importlib.util import spec_from_file_location,module_from_spec
spec=spec_from_file_location('three_checker','/home/ubuntu/mip-codex/artifacts/objective-box-report-v1-diagnostic/scripts/verify_cover_cuts.py');check_module=module_from_spec(spec);spec.loader.exec_module(check_module);validate_cut_proofs=check_module.validate_cut_proofs
os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';os.sched_setaffinity(0,{40});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3));root=Path.cwd();source=root/'artifacts/objective-box-report-v1-development-screen';out=root/'artifacts/objective-box-report-v1-development-audit';out.mkdir(exist_ok=False);manifest={r['name']:r for r in json.loads((root/'data/manifest.json').read_text())['instances']};records=[];seen=set();atomic_json(out/'experiment.json',dict(source=str(source),cpu=40,memory_bytes=16*1024**3,driver_sha256=sha256(Path(__file__)),cut_checker_sha256=sha256(root/'artifacts/objective-box-report-v1-diagnostic/scripts/verify_cover_cuts.py'),checker_sha256=sha256(root/'third_party/miplib-checker/bin/solchecker-absolute')))
while True:
 for p in sorted(source.glob('cases/*/*/summary.json')):
  key=str(p)
  if key in seen:continue
  summary=json.loads(p.read_text());name=summary['name'];r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];result=p.parent/'result.json';destination=out/name/summary['variant'];destination.mkdir(parents=True);record=dict(name=name,variant=summary['variant'],solver_status=summary['status'],input_sha256=r['sha256'],cut_validation=None,strict=None,official=None)
  if result.exists():
   value=json.loads(result.read_text());record['result_sha256']=sha256(result)
   try:record['cut_validation']=validate_cut_proofs(model,value)
   except Exception as error:record['cut_validation']=dict(valid=False,error=f'{type(error).__name__}: {error}')
   if value.get('values')is not None:
    cmd=[str(root/'.venv/bin/python'),'/home/ubuntu/mip-codex/scripts/benchmark_validate.py',str(model),str(result),str(root/'third_party/miplib-checker/bin/solchecker'),str(destination)];start=time.monotonic()
    with (destination/'audit.log').open('w')as stream:
     proc=subprocess.Popen(cmd,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
     try:proc.wait(timeout=600)
     except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait();record['watchdog']=True
     except BaseException:
      if proc.poll()is None:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
      raise
    record.update(validation_exit_code=proc.returncode,validation_wall_seconds=time.monotonic()-start)
    for kind in ['strict','official']:
     q=destination/(kind+'-validation.json');record[kind]=json.loads(q.read_text())if q.exists()else dict(valid=None,error='validation output missing')
  else:record['reason']='solver driver produced no valid JSON result'
  atomic_json(destination/'audit-record.json',record);records.append(record);seen.add(key);print(json.dumps({k:v for k,v in record.items()if k not in ['strict','official','cut_validation']},ensure_ascii=False),flush=True)
  atomic_json(out/'records.json',records);stats=dict(completed=len(records),cut_valid=sum(x['cut_validation']is not None and x['cut_validation']['valid']is True for x in records),cut_invalid=sum(x['cut_validation']is not None and x['cut_validation']['valid']is False for x in records),points=sum(x['strict']is not None for x in records),strict_valid=sum(x['strict']is not None and x['strict']['valid']is True for x in records),strict_invalid=sum(x['strict']is not None and x['strict']['valid']is False for x in records),strict_unverified=sum(x['strict']is not None and x['strict']['valid']is None for x in records),official_valid=sum(x['official']is not None and x['official']['valid']is True for x in records));atomic_json(out/'summary.json',stats)
 exits=list(source.glob('worker-*-exit.json'))
 if len(exits)==5 and len(records)==len(list(source.glob('cases/*/*/summary.json'))):
  worker_errors=any(json.loads(p.read_text())['status']!='complete'for p in exits);failed=worker_errors or len(records)!=376 or stats['cut_invalid']or stats['strict_invalid']or stats['strict_unverified']or stats['points']!=stats['official_valid'];atomic_json(out/'exit.json',dict(status='failed'if failed else'complete',records=len(records),worker_errors=worker_errors));break
 time.sleep(1)
