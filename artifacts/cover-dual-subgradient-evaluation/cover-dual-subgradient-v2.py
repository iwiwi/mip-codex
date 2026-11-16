from pathlib import Path
import subprocess,json,hashlib,shutil
root=Path.cwd();out=root/'artifacts/cover-dual-subgradient-v2';out.mkdir();binary=root/'target-dual-restoration/release/examples/cover_dual_subgradient';shutil.copy2('/home/ubuntu/mip-dual-restoration/examples/cover_dual_subgradient.rs',out/'source.rs');(out/'binary.sha256').write_text(hashlib.sha256(binary.read_bytes()).hexdigest()+'\n');records=[]
for factor in ['0.1','0.5','1','2']:
 p=out/factor;p.mkdir();cmd=['taskset','-c','42','timeout','--signal=KILL','65',str(binary),str(root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz'),str(root/'artifacts/root-objective-repair-v1-paired-41/neos-5052403-cygnet.mps.gz/after/result.json'),'2',factor]
 with(p/'stdout.json').open('w')as stdout,(p/'stderr.log').open('w')as stderr:r=subprocess.run(cmd,stdout=stdout,stderr=stderr)
 assert r.returncode==0;a=json.load(open(p/'stdout.json'));records.append(dict(factor=factor,**{k:v for k,v in a.items()if k!='multipliers'}));(out/'summary.json').write_text(json.dumps(records,indent=2))
print(records)
