from pathlib import Path
import json,subprocess,hashlib,os,sys
sys.path.insert(0,'scripts')
from benchmark_common import atomic_json,sha256
root=Path.cwd();out=root/'artifacts/binary-equality-inventory-20261117';out.mkdir();os.sched_setaffinity(0,{39});binary=Path('/home/ubuntu/mip-objective-repair/target/release/examples/equality_inventory');source=Path('/home/ubuntu/mip-objective-repair/examples/equality_inventory.rs');(out/'helper.rs').write_bytes(source.read_bytes());atomic_json(out/'experiment.json',dict(purpose='read-only structural development inventory; not optimization',binary_sha256=sha256(binary),helper_sha256=sha256(source),library_build_info=json.loads(subprocess.check_output(['/home/ubuntu/mip-objective-repair/target/release/mip-codex','build-info'])),cpu=39));records=[]
manifest=[r for r in json.loads((root/'data/manifest.json').read_text())['instances']if r['split']=='development'];assert len(manifest)==188
for r in manifest:
 model=root/r['path'];assert sha256(model)==r['sha256'];d=out/r['name'];d.mkdir()
 with(d/'stdout.json').open('w')as stream,(d/'stderr.log').open('w')as err:p=subprocess.run([str(binary),str(model)],stdout=stream,stderr=err,timeout=60)
 record=dict(name=r['name'],input_sha256=r['sha256'],exit_code=p.returncode)
 if p.returncode==0:record['structure']=json.loads((d/'stdout.json').read_text())
 records.append(record);atomic_json(d/'record.json',record);atomic_json(out/'records.json',records)
atomic_json(out/'summary.json',dict(cases=len(records),success=sum(r['exit_code']==0 for r in records),applicable=sum(r.get('structure',{}).get('redundant_columns',0)>0 for r in records)));atomic_json(out/'exit.json',dict(status='complete'));print(json.loads((out/'summary.json').read_text()))
