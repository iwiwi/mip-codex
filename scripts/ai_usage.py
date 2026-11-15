#!/usr/bin/env python3
"""Sum final cumulative token counters per local session; never sum snapshots."""
import datetime,json,pathlib
root=pathlib.Path.home()/'.codex/sessions';records=[]
for p in root.rglob('*.jsonl'):
 last=None
 for line in p.open():
  try: event=json.loads(line)
  except json.JSONDecodeError: continue
  payload=event.get('payload',{})
  if payload.get('type')=='token_count' and payload.get('info'):
   total=payload['info'].get('total_token_usage')
   if total: last=total
 if last: records.append(dict(session=str(p.relative_to(root)),usage=last))
keys=sorted({k for r in records for k in r['usage']})
print(json.dumps(dict(recorded_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),scope='All locally available session cumulative counters; cached input is included in input, not added twice; excludes unlogged usage.',totals={k:sum(r['usage'].get(k,0) for r in records) for k in keys},sessions=records),indent=2))
