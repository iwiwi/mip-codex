from pathlib import Path
import urllib.request,json,hashlib,gzip,datetime,os
os.sched_setaffinity(0,{43});root=Path.cwd();assert 'z26.mps.gz'not in{r['name']for r in json.load(open('data/manifest.json'))['instances']};url='https://miplib.zib.de/WebData/instances/z26.mps.gz';target=root/'data/raw/exploratory/z26.mps.gz';temporary=root/'artifacts/large-packing-inputs/z26.mps.gz.download';assert not temporary.exists();request=urllib.request.Request(url,headers={'User-Agent':'mip-codex research input download'})
with urllib.request.urlopen(request,timeout=60)as response,temporary.open('wb')as stream:
 assert response.status==200;headers=dict(response.headers)
 while True:
  block=response.read(1024*1024)
  if not block:break
  stream.write(block);assert stream.tell()<300*1024**2
compressed=hashlib.sha256(temporary.read_bytes()).hexdigest();uncompressed=hashlib.sha256();size=0
with gzip.open(temporary,'rb')as stream:
 while True:
  block=stream.read(1024*1024)
  if not block:break
  size+=len(block);assert size<2*1024**3;uncompressed.update(block)
if target.exists():assert hashlib.sha256(target.read_bytes()).hexdigest()==compressed;temporary.unlink()
else:target.parent.mkdir(parents=True,exist_ok=True);temporary.rename(target)
record=dict(time=datetime.datetime.now(datetime.timezone.utc).isoformat(),url=url,local_path=str(target.relative_to(root)),gzip_sha256=compressed,uncompressed_sha256=uncompressed.hexdigest(),gzip_bytes=target.stat().st_size,uncompressed_bytes=size,headers=headers,split='exploratory; outside Benchmarkv2; not in240',solution_downloaded=False,optimizer_target_or_stop_value=None);Path('artifacts/large-packing-inputs/z26-source.json').write_text(json.dumps(record,indent=2));print(json.dumps({k:v for k,v in record.items()if k!='headers'}))
