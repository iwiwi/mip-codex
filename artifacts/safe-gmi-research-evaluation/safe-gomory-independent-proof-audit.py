from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
import json,copy,os,sys
sys.path.insert(0,"scripts")
root=Path.cwd();out=root/'artifacts/safe-gomory-independent-proof-audit';out.mkdir();os.sched_setaffinity(0,{42});spec=spec_from_file_location('checker','/home/ubuntu/mip-dual-restoration/scripts/verify_cover_cuts.py');checker=module_from_spec(spec);spec.loader.exec_module(checker);inputs=json.load(open('artifacts/safe-gomory-rust-oracle/input.json'));rows=[json.loads(x)for x in Path('artifacts/safe-gomory-rust-oracle/rust.jsonl').read_text().splitlines()];valid=0;tampered=0;records=[]
for index,(v,row)in enumerate(zip(inputs,rows)):
 if row is None:continue
 d=out/str(index);d.mkdir();model=d/'model.mps';model.write_text(v['model']);first=v['proof']['terms'][0];names,variables,source=checker.exact_model(model);p=dict(gomory=v['proof'],source_row=first['row'],source_name=source[first['row']]['name'],upper_side=first['upper'],cover=[],clique=[],aggregation=[],divisor=None);cut=dict(row=row,proof=p);result=dict(cut_proofs=[cut]);good=checker.validate_cut_proofs(model,result);assert good['valid']is True,(index,good);valid+=1
 bad=copy.deepcopy(result);bad['cut_proofs'][0]['row']['coefficients'][0][1]+=1;check=checker.validate_cut_proofs(model,bad);assert check['valid']is False,(index,check);tampered+=1;records.append(dict(case=index,valid=good,tampered_rejected=check))
(out/'records.json').write_text(json.dumps(records,indent=2)+'\n');summary=dict(valid=valid,tampered_rejected=tampered,failures=0);(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(summary)
