from pathlib import Path
import os,json,subprocess,signal
root=Path.cwd();source=root/'artifacts/dual-guided-repair-diagnostic';out=root/'artifacts/dual-guided-repair-diagnostic-audit';out.mkdir();env=dict(os.environ,MIP_ABSOLUTE_CHECKER=str(root/'third_party/miplib-checker/bin/solchecker-absolute'));records=[]
for p in sorted(source.glob('*/*/solution.json')):
 name=p.parent.parent.name;d=out/name/p.parent.name;d.mkdir(parents=True);cmd=[str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(root/'data/raw/benchmark'/name),str(p),str(root/'third_party/miplib-checker/bin/solchecker'),str(d)]
 with(d/'audit.log').open('w')as stream:
  proc=subprocess.Popen(cmd,env=env,stdout=stream,stderr=subprocess.STDOUT,start_new_session=True)
  try:proc.wait(timeout=600)
  except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait()
 checks={k:json.load(open(d/f'{k}-validation.json'))for k in ['strict','official']};assert proc.returncode==0 and all(v['valid']is True for v in checks.values());records.append(dict(source=str(p),checks=checks));(out/'progress.json').write_text(json.dumps(dict(records=len(records),valid=len(records))))
(out/'records.json').write_text(json.dumps(records,indent=2));(out/'summary.json').write_text(json.dumps(dict(records=len(records),valid=len(records)),indent=2));print(len(records))
