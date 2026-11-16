from pathlib import Path
import subprocess,json,shutil,sys
sys.path.insert(0,'scripts');from benchmark_common import sha256
root=Path.cwd();out=root/'artifacts/priced-portfolio-diagnostic';out.mkdir();binary=root/'artifacts/priced-primal-repair-v1-diagnostic/bin/priced_repair';(out/'experiment.json').write_text(json.dumps(dict(binary=str(binary),sha256=sha256(binary),seconds=.5,cpu=42,purpose='additional global weight settings; no problem-specific coefficients'),indent=2));records=[]
for name in ['n3div36.mps.gz','sp97ar.mps.gz','sp98ar.mps.gz','tbfp-network.mps.gz']:
 seed=root/'artifacts/priced-root-repair-development-screen/cases'/name/'before/result.json';model=root/'data/raw/benchmark'/name
 for weight in ['0.3','3']:
  d=out/name/weight;d.mkdir(parents=True);cmd=['taskset','-c','42','timeout','--signal=KILL','65',str(binary),str(model),str(seed),'0.5',weight,str(d/'solution.json')];(d/'input.json').write_text(json.dumps(dict(input_sha256=sha256(model),seed_sha256=sha256(seed)),indent=2))
  with(d/'stdout.json').open('w')as stdout,(d/'stderr.log').open('w')as stderr:p=subprocess.run(cmd,stdout=stdout,stderr=stderr)
  assert p.returncode==0;a=json.load(open(d/'stdout.json'));records.append(dict(name=name,**a));(out/'summary.json').write_text(json.dumps(records,indent=2))
print(records)
