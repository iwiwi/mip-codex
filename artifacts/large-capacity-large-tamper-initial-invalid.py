from pathlib import Path
import os,sys,json,copy
os.sched_setaffinity(0,{43});root=Path.cwd();sys.path.insert(0,'scripts');from benchmark_common import atomic_json;from verify_cover_cuts import validate_cut_proofs
source=root/'artifacts/large-capacity-generated';assert json.load(open(source/'summary.json'))['outputs']==40;out=root/'artifacts/large-capacity-large-tamper';out.mkdir();records=[]
for d in sorted(source.iterdir()):
 if not d.is_dir():continue
 model=d/'model.mps';r=json.load(open(d/'result.json'))
 for mode in ['capacity','bound','color','permutation']:
  bad=copy.deepcopy(r)
  if mode=='capacity':bad['binary_graph']['vertices']=65536
  elif mode=='bound':bad['dual_bound']+=-1 if r['maximize']else 1
  elif mode=='color':bad['binary_graph']['root_colors'][0]=0
  else:bad['binary_graph']['order'][1]=bad['binary_graph']['order'][0]
  dest=out/d.name/mode;dest.mkdir(parents=True);atomic_json(dest/'result.json',bad);v=validate_cut_proofs(model,bad);atomic_json(dest/'validation.json',v);assert v['valid']is False,(d,mode,v);records.append(dict(case=d.name,mode=mode,rejected=True))
atomic_json(out/'summary.json',dict(tamper_rejected=len(records),cases=40));print(len(records),flush=True)
