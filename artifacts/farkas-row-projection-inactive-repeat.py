from pathlib import Path
import os,sys,json,subprocess,resource
root=Path.cwd();os.sched_setaffinity(0,{52});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ.update(OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1',MIP_ABSOLUTE_CHECKER='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute');sys.path.insert(0,'scripts');from benchmark_common import atomic_json,sha256;from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/farkas-row-projection-inactive-repeat';out.mkdir();binary=root/'artifacts/farkas-row-projection-frozen/bin/mip-codex';model=root/'data/raw/benchmark/n3div36.mps.gz';records=[]
for repeat in range(3):
 for flag in ([0,1]if repeat%2==0 else[1,0]):
  d=out/str(repeat)/str(flag);d.mkdir(parents=True);env={k:v for k,v in os.environ.items()if not k.startswith('MIP_')};env.update(MIP_LP_FARKAS_ROW_PROJECT=str(flag),OMP_NUM_THREADS='1',OPENBLAS_NUM_THREADS='1');cmd=[str(binary),'solve',str(model),'10'];atomic_json(d/'job.json',dict(command=cmd,cpu=52,binary_sha256=sha256(binary),input_sha256=sha256(model),environment={k:v for k,v in env.items()if k.startswith('MIP_')},purpose='three predetermined pairs; original188 loss retained; never replace original result'));r=subprocess.run(cmd,env=env,capture_output=True,text=True,timeout=70);(d/'stdout.log').write_text(r.stdout);(d/'stderr.log').write_text(r.stderr);assert r.returncode==0;v=json.loads(r.stdout);atomic_json(d/'result.json',v);proof=validate_cut_proofs(model,v);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True;point=None
  if v.get('values')is not None:
   audit=d/'point-audit';audit.mkdir()
   with(audit/'audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(d/'result.json'),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(audit)],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=600)
   for kind in ['strict','official']:assert json.load(open(audit/f'{kind}-validation.json'))['valid']is True
   point=True
  record=dict(repeat=repeat,flag=flag,status=v['status'],objective=v['objective'],dual_bound=v['dual_bound'],trials=v.get('lp_farkas_row_projection_trials'),projections=v.get('lp_farkas_row_projections'),point_valid=point,proof_valid=True);records.append(record);atomic_json(out/'records.json',records);print(json.dumps(record),flush=True)
atomic_json(out/'exit.json',dict(status='complete',outputs=6,original_loss_retained=True,original_not_replaced=True))
