from pathlib import Path
import concurrent.futures as cf,subprocess,json,time,resource,hashlib
root=Path.cwd();out=root/'artifacts/row-ray-bound-development';out.mkdir(exist_ok=True);binary=root/'target-dual-restoration/release/examples/row_ray_bound';names=(root/'data/development.txt').read_text().splitlines();assert len(names)==188
(out/'experiment.json').write_text(json.dumps(dict(binary=str(binary),sha256=hashlib.sha256(binary.read_bytes()).hexdigest(),seconds=60,cpus=[41,42],memory_bytes=16*1024**3,instances=names),indent=2))
def limit():resource.setrlimit(resource.RLIMIT_AS,(16*1024**3,16*1024**3))
def shard(pair):
 index,cpu=pair;records=[]
 for name in names[index::2]:
  p=out/name;p.mkdir(exist_ok=True);start=time.monotonic()
  with (p/'stdout.json').open('w') as stdout,(p/'stderr.log').open('w') as stderr:
   c=subprocess.run(['taskset','-c',str(cpu),'timeout','--signal=KILL','60',str(binary),str(root/'data/raw/benchmark'/name)],stdout=stdout,stderr=stderr,preexec_fn=limit)
  record=dict(name=name,returncode=c.returncode,wall=time.monotonic()-start)
  if c.returncode==0:
   a=json.load(open(p/'stdout.json'));record.update(supported=a['supported'],bound=a.get('normalized_bound'),seconds=a.get('seconds'),rows=len(a.get('rows',[])))
  (p/'execution.json').write_text(json.dumps(record));records.append(record);(out/f'shard-{index}.json').write_text(json.dumps(records,indent=2))
 return records
with cf.ThreadPoolExecutor(2) as pool:records=sum(list(pool.map(shard,enumerate([41,42]))),[])
(out/'summary.json').write_text(json.dumps(dict(completed=len(records),supported=sum(a.get('supported',False) for a in records),records=records),indent=2));print('completed',len(records))
