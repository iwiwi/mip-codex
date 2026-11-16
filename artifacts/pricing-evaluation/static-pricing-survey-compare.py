import collections,json
from pathlib import Path
root=Path('artifacts');records={}
for mode in ['baseline','guarded','restart']:
 d=root/f'pricing-{mode}-root-survey';items={p.parent.name:json.loads(p.read_text())for p in d.glob('*/summary.json')};records[mode]=items
 if len(items)!=188:continue
 audits=[json.loads(p.read_text())for p in d.glob('analysis-partition-*.json')]
 summary=dict(models=len(items),points=sum(r['point_dump']for r in items.values()),statuses=dict(collections.Counter(r.get('status','driver_error')for r in items.values())),driver_errors=sum('diagnostic_error'in r for r in items.values()),watchdogs=sum(r['execution'].get('external_watchdog',False)for r in items.values()),audit_parts=len(audits),valid=sum(r['valid']for r in audits),invalid=sum(r['invalid']for r in audits))
 (d/'analysis-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(mode,summary)
a=records['baseline']
for mode,b in records.items():
 if mode=='baseline'or len(b)!=188:continue
 gained=[n for n in a if b[n]['point_dump']and not a[n]['point_dump']];lost=[n for n in a if a[n]['point_dump']and not b[n]['point_dump']];common=[n for n in a if a[n]['point_dump']and b[n]['point_dump']];r=dict(variant=mode,gained=gained,lost=lost,common_points=len(common),paired=[dict(name=n,before={k:a[n].get(k)for k in ['status','seconds','lp_pivots','point_dump']},after={k:b[n].get(k)for k in ['status','seconds','lp_pivots','point_dump']})for n in gained+lost]);(root/f'static-pricing-{mode}-survey-comparison.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r,indent=2))
