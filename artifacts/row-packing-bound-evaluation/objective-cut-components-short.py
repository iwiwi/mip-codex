from pathlib import Path
import subprocess,json,time
root=Path.cwd();out=root/'artifacts/objective-cut-components-short';out.mkdir();records=[]
for seconds in ['0.5','1','2','10']:
 d=out/seconds;d.mkdir();cmd=['taskset','-c','42','timeout','--signal=KILL','75',str(root/'target-dual-restoration/release/examples/objective_cut_components'),str(root/'data/raw/benchmark/neos-5052403-cygnet.mps.gz'),str(root/'artifacts/row-packing-bound-v2-paired-41/neos-5052403-cygnet.mps.gz/after/result.json'),seconds,str(d/'solution.json')];start=time.monotonic()
 with (d/'stdout.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:p=subprocess.run(cmd,stdout=stdout,stderr=stderr)
 assert p.returncode==0;a=json.load(open(d/'stdout.json'));records.append(dict(seconds=seconds,objective=a['objective'],reported_seconds=a['seconds'],wall=time.monotonic()-start));(out/'summary.json').write_text(json.dumps(records,indent=2))
print(records)
