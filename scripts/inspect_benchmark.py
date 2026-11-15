#!/usr/bin/env python3
"""Persist each parser result immediately; a terminal summary proves completion."""
import argparse,hashlib,json,os,pathlib,subprocess,time
ROOT=pathlib.Path(__file__).resolve().parents[1];p=argparse.ArgumentParser();p.add_argument('--output',required=True);p.add_argument('--cpu',type=int,default=2);args=p.parse_args();out=pathlib.Path(args.output);out.mkdir(parents=True,exist_ok=True);assert not (out/'results.jsonl').exists()
os.sched_setaffinity(0,{args.cpu});binary=ROOT/'target/release/mip-codex';manifest=json.loads((ROOT/'docs/benchmark-manifest.json').read_text());start=time.monotonic();results=[]
with (out/'results.jsonl').open('w') as log:
 for row in manifest['instances']:
  run=subprocess.run([str(binary),'inspect',str(ROOT/row['path'])],capture_output=True,text=True,timeout=300)
  result=dict(instance=row['name'],code=run.returncode,stderr=run.stderr)
  if run.returncode==0:result['model']=json.loads(run.stdout)
  results.append(result);log.write(json.dumps(result)+'\n');log.flush()
summary=dict(status='finished',count=len(results),passed=sum(r['code']==0 for r in results),seconds=time.monotonic()-start,binary_sha256=hashlib.sha256(binary.read_bytes()).hexdigest())
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
