#!/usr/bin/env python3
"""全MPSの目的整数性だけを検査する。LP/MIP最適化や性能調整はしない。"""
import json,subprocess,time
from pathlib import Path
from benchmark_common import sha256
from verify_cover_cuts import validate_cut_proofs
root=Path(__file__).resolve().parents[1]
manifest=json.loads((root/'data/manifest.json').read_text());failed=0
for record in manifest['instances']:
    started=time.monotonic();path=root/record['path']
    try:
        assert sha256(path)==record['sha256'],'input hash mismatch'
        result=json.loads(subprocess.check_output([str(root/'target/release/mip-codex'),'objective-proof',str(path)],stderr=subprocess.PIPE))
        check=validate_cut_proofs(path,result)
        assert check['valid']is True,check
        print(json.dumps(dict(name=record['name'],status='pass',has_proof=result['objective_grid']is not None,validation=check,seconds=time.monotonic()-started)),flush=True)
    except Exception as error:
        failed+=1;print(json.dumps(dict(name=record['name'],status='fail',error=f'{type(error).__name__}: {error}')),flush=True)
print(json.dumps(dict(files=len(manifest['instances']),failures=failed)))
raise SystemExit(bool(failed))
