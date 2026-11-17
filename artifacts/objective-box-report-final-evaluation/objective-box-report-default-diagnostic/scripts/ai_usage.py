#!/usr/bin/env python3
"""各セッションの最後の累積usageを一度だけ加算。秘密情報や会話本文は出力しない。"""
import json
from pathlib import Path
usage = {}
for path in Path('/home/ubuntu/.codex/sessions').rglob('*.jsonl'):
    last = None
    for line in path.open():
        try:
            record = json.loads(line)
        except json.JSONDecodeError:
            continue
        payload = record.get('payload', {})
        if record.get('type') == 'event_msg' and payload.get('type') == 'token_count':
            info = payload.get('info') or {}
            last = info.get('total_token_usage') or last
    if last:
        usage[path.name] = last
keys = ('input_tokens','cached_input_tokens','output_tokens','reasoning_output_tokens','total_tokens')
print(json.dumps(dict(sessions=len(usage), totals={k:sum(x.get(k,0) for x in usage.values()) for k in keys},
                     note='input_tokensはキャッシュ分を含む。reasoningはoutputの内数。ローカルログの最終記録まで。'), ensure_ascii=False, indent=2))
