import json,sys,os,subprocess,shutil,resource,time,signal
from pathlib import Path
sys.path.insert(0,'scripts');from benchmark_common import sha256,atomic_json
root=Path.cwd();work=Path('/home/ubuntu/mip-dual-restoration');out=root/'artifacts/exact-row-dominance-inventory-v1';out.mkdir(exist_ok=False);(out/'bin').mkdir();binary=work/'target/release/examples/row_dominance_inventory';shutil.copy2(binary,out/'bin/row_dominance_inventory');shutil.copytree(work/'src',out/'src')
for n in ['Cargo.toml','Cargo.lock','build.rs']:shutil.copy2(work/n,out/n)
shutil.copy2(work/'examples/row_dominance_inventory.rs',out/'example.rs');binary=out/'bin/row_dominance_inventory';os.sched_setaffinity(0,{0});manifest={r['name']:r for r in json.load(open('data/manifest.json'))['instances']};records=[];atomic_json(out/'experiment.json',dict(purpose='development-only exact row dominance detection, parser/proof generation only; no optimization',cpu=0,binary_sha256=sha256(binary),example_sha256=sha256(out/'example.rs'),memory_bytes=16*1024**3))
def restrict():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
for name in ['ns1208400.mps.gz','neos-860300.mps.gz','neos-3046615-murg.mps.gz','neos-3656078-kumeu.mps.gz','netdiversion.mps.gz','neos-957323.mps.gz','uccase12.mps.gz','physiciansched6-2.mps.gz']:
 r=manifest[name];assert r['split']=='development';model=root/r['path'];assert sha256(model)==r['sha256'];d=out/name;d.mkdir();start=time.monotonic()
 with(d/'result.json').open('w')as f,(d/'stderr.log').open('w')as e:
  p=subprocess.Popen([str(binary),str(model)],stdout=f,stderr=e,preexec_fn=restrict,start_new_session=True)
  try:p.wait(timeout=60)
  except BaseException:
   if p.poll()is None:os.killpg(p.pid,signal.SIGKILL);p.wait()
   raise
 assert p.returncode==0;v=json.load(open(d/'result.json'));r=dict(name=name,input_sha256=r['sha256'],rows=v['rows'],source_fingerprint=v['source_fingerprint'],stats=v['stats'],seconds=time.monotonic()-start);records.append(r);print(r,flush=True);atomic_json(out/'records.json',records)
atomic_json(out/'exit.json',dict(status='complete',cases=len(records)))
