#!/usr/bin/env python3
"""Fetch and hash the official Benchmark Set v2; input metadata is evaluation-only."""
import gzip,hashlib,json,pathlib,urllib.request,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[1]
BASE='https://miplib.zib.de/downloads/'
def sha(data):return hashlib.sha256(data).hexdigest()
def main():
 data=ROOT/'data';data.mkdir(exist_ok=True);raw=data/'benchmark';raw.mkdir(exist_ok=True)
 for name in ('benchmark.zip','benchmark-v2.test'):
  path=data/name
  if not path.exists():urllib.request.urlretrieve(BASE+name,path)
 names=(data/'benchmark-v2.test').read_text().split();assert len(names)==len(set(names))==240
 records=[]
 with zipfile.ZipFile(data/'benchmark.zip') as archive:
  assert set(archive.namelist())==set(names)
  for name in sorted(names):
   compressed=archive.read(name);expanded=gzip.decompress(compressed)
   target=raw/name
   if not target.exists() or sha(target.read_bytes())!=sha(compressed):
    tmp=target.with_suffix(target.suffix+'.tmp');tmp.write_bytes(compressed);tmp.replace(target)
   records.append(dict(name=name.removesuffix('.mps.gz'),path='data/benchmark/'+name,compressed_sha256=sha(compressed),mps_sha256=sha(expanded)))
 ordered=sorted(records,key=lambda r:sha(r['name'].encode()))
 for i,r in enumerate(ordered):r['split']='development' if i<160 else 'validation'
 manifest=dict(source=BASE+'benchmark.zip',archive_sha256=sha((data/'benchmark.zip').read_bytes()),list_sha256=sha((data/'benchmark-v2.test').read_bytes()),instances=records)
 (ROOT/'docs/benchmark-manifest.json').write_text(json.dumps(manifest,indent=2)+'\n')
 (ROOT/'docs/benchmark-v2.test').write_bytes((data/'benchmark-v2.test').read_bytes())
 for split in ('development','validation'):(ROOT/'docs'/f'{split}.test').write_text(''.join(r['name']+'\n' for r in ordered if r['split']==split))
 print(json.dumps(dict(count=len(records),archive_sha256=manifest['archive_sha256'])))
if __name__=='__main__':main()
