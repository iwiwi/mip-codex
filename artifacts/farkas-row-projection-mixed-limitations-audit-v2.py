from pathlib import Path
import os,sys,json,subprocess,resource
root=Path.cwd();os.sched_setaffinity(0,{52});resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,)*2);os.environ['MIP_ABSOLUTE_CHECKER']='/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker-absolute';sys.path.insert(0,'scripts');from benchmark_common import atomic_json;from verify_cover_cuts import validate_cut_proofs
out=root/'artifacts/farkas-row-projection-mixed-limitations-audit-v2';out.mkdir();records=[]
for label in ['mixed-off','mixed-on']:
 source=root/'artifacts/farkas-row-projection-synthetic'/label;summary=json.load(open(source/'summary.json'));assert [r['case']for r in summary['failures']]==[8,40,87,122,169]
 for case in [8,40,87,122,169]:
  d=out/label/str(case);d.mkdir(parents=True);model=source/f'{case:04d}'/'model.mps';result=source/f'{case:04d}'/'result.json';r=json.load(open(result));assert r['status']=='numerical_error'and r['dual_bound']is None;proof=validate_cut_proofs(model,r);atomic_json(d/'proof-validation.json',proof);assert proof['valid']is True
  with(d/'audit.log').open('w')as f:subprocess.run([str(root/'.venv/bin/python'),'scripts/benchmark_validate.py',str(model),str(result),'/home/ubuntu/mip-codex/third_party/miplib-checker/bin/solchecker',str(d)],stdout=f,stderr=subprocess.STDOUT,check=True,timeout=600)
  strict=json.load(open(d/'strict-validation.json'));official=json.load(open(d/'official-validation.json'));assert strict['valid']is True
  parser_unsupported=False
  if official['valid']is None:
   text=(d/'official-checker.log').read_text();parser_unsupported='Syntax error'in text and 'Read MPS: 0'in text
  records.append(dict(label=label,case=case,reported_status='numerical_error',strict_valid=True,official_valid=official['valid'],official_parser_unsupported=parser_unsupported,proof_valid=True));atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',outputs=10,strict_valid=10,proof_valid=10,official_valid=sum(r['official_valid']is True for r in records),official_invalid=sum(r['official_valid']is False for r in records),official_parser_unsupported=sum(r['official_parser_unsupported']for r in records),optimizer_not_rerun=True))
