from pathlib import Path
import os,json,subprocess,sys
root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256
prior=Path('/home/ubuntu/mip-graph-vertex-portfolio/artifacts')
for name in ['development-screen','parallel-audit','development-analysis']:
 s=(prior/f'graph-vertex-{name}.py').read_text().replace('graph-vertex-', 'farkas-row-projection-').replace('MIP_GRAPH_VERTEX_PORTFOLIO','MIP_LP_FARKAS_ROW_PROJECT').replace("['graph-vertex']","['farkas-row-projection']")
 if name=='development-screen':s=s.replace("summary['graph_recognized']=", "summary['lp_farkas_row_projection_trials']=value.get('lp_farkas_row_projection_trials');summary['lp_farkas_row_projections']=value.get('lp_farkas_row_projections');summary['graph_recognized']=")
 if name=='development-analysis':
  s=s.replace("'seconds','binary_graph']","'seconds','binary_graph','lp_farkas_row_projection_trials','lp_farkas_row_projections']")
  s=s.replace("((r['after']['binary_graph']or{}).get('vertex_local')or{}).get('used',False)","(r['after'].get('lp_farkas_row_projections')or 0)>0")
 (root/'artifacts'/f'farkas-row-projection-{name}.py').write_text(s)
out=root/'artifacts/farkas-row-projection-development-screen';out.mkdir();origin=json.load(open('artifacts/farkas-row-projection-origin.json'));binary=root/'artifacts/farkas-row-projection-frozen/bin/mip-codex';record=dict(path=str(binary),sha256=sha256(binary),build_info=origin['build_info']);atomic_json(out/'experiment.json',dict(shards=5,binaries={k:record for k in ['before','after']},seconds=10,threads=1,memory_bytes=16*1024**3,cpus=[31,32,33,34,35],farkas_row_projection={'before':'explicit0','after':'explicit1; defaultOFF'},purpose='development188 only; same frozen CLI Farkas row projection explicit0/1; all other defaults main133; no holdout quality read; independent strict/official/proof audits'))
workers=[]
for shard,cpu in enumerate([31,32,33,34,35]):
 with(out/f'worker-{shard}.log').open('w')as log:p=subprocess.Popen([sys.executable,'artifacts/farkas-row-projection-development-screen.py',str(cpu),str(shard),'5'],stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
 workers.append(dict(shard=shard,cpu=cpu,pid=p.pid))
atomic_json(out/'workers.json',workers)
env=os.environ.copy();env['MIP_AUDIT_CPUS']='36,37,48,49';log=(root/'artifacts/farkas-row-projection-parallel-audit.log').open('w');p=subprocess.Popen([sys.executable,'artifacts/farkas-row-projection-parallel-audit.py','farkas-row-projection'],stdout=log,stderr=subprocess.STDOUT,env=env,start_new_session=True);atomic_json(out/'audit-process.json',dict(pid=p.pid));print(json.dumps(workers))
