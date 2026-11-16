from pathlib import Path
import os,subprocess,json,signal
root=Path.cwd();out=root/'artifacts/objective-cut-diagnostic-audit';out.mkdir();env=dict(os.environ,MIP_ABSOLUTE_CHECKER=str(root/'third_party/miplib-checker/bin/solchecker-absolute'));records=[]
paths=sorted(root.glob('artifacts/objective-cut-repair-short/*/solution.json'))+sorted(root.glob('artifacts/objective-cut-components-short/*/solution.json'))
for index,p in enumerate(paths):
 d=out/f'{index:02}';d.mkdir();cmd=[str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz'),str(p),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]
 with(d/'audit.log').open('w')as stream:
  proc=subprocess.Popen(cmd,stdout=stream,stderr=subprocess.STDOUT,env=env,start_new_session=True)
  try:proc.wait(timeout=600)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
  except BaseException:
   if proc.poll()is None:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
   raise
 checks={k:json.load(open(d/f'{k}-validation.json'))for k in ['strict','official']};assert proc.returncode==0 and all(c['valid']is True for c in checks.values());records.append(dict(source=str(p),checks=checks));(out/'progress.json').write_text(json.dumps(dict(records=len(records),valid=len(records)),indent=2))
(out/'records.json').write_text(json.dumps(records,indent=2));(out/'summary.json').write_text(json.dumps(dict(records=len(records),valid=len(records)),indent=2));print(len(records))
