from pathlib import Path
import subprocess,os,sys,json
root=Path.cwd();jobs=[]
for name,args in [('main-controls',[]),('long-controls',[]),('large-generated',[]),('parallel-audit',['cover-long-round-default'])]:
 log=root/f'artifacts/cover-long-round-default-{name}.log'
 with log.open('w')as stream:child=subprocess.Popen([sys.executable,str(root/f'artifacts/cover-long-round-default-{name}.py'),*args],stdout=stream,stderr=subprocess.STDOUT,start_new_session=True,env=dict(os.environ,MIP_AUDIT_CPUS='36,37,42,48'))
 jobs.append(dict(name=name,pid=child.pid,log=str(log)))
with(root/'artifacts/cover-long-round-default-enumeration.log').open('w')as stream:
 child=subprocess.Popen(['taskset','-c','41',sys.executable,'scripts/check_binary_graph.py','200'],stdout=stream,stderr=subprocess.STDOUT,start_new_session=True,env=dict(os.environ,MIP_GRAPH_DIR='artifacts/cover-long-round-default-enumeration',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute'))
jobs.append(dict(name='enumeration',pid=child.pid));(root/'artifacts/cover-long-round-default-workers.json').write_text(json.dumps(jobs,indent=2));print(json.dumps(jobs),flush=True)
