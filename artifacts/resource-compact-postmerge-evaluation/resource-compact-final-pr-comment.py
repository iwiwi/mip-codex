from pathlib import Path
import json,subprocess
r=Path.cwd();repo='iwiwi/mip-codex';head='bd50d8d20d3b25e9e19a4b6506a153dff87411cd'
a=json.loads((r/'artifacts/resource-compact-default-development-audit/summary.json').read_text());assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==128
archive=json.loads(Path('/home/ubuntu/mip-phase-two/artifacts/resource-compact-final-evaluation/archive-summary.json').read_text());assert archive['files']==archive['verified_payloads']==14726
review=json.loads(subprocess.check_output(['gh','pr','view','116','--repo',repo,'--json','headRefOid,baseRefName,isDraft,mergeable,statusCheckRollup'],text=True));assert review['headRefOid']==head and review['baseRefName']=='main'and not review['isDraft']and not review['statusCheckRollup'];assert review['mergeable']=='MERGEABLE',review
(r/'artifacts/resource-compact-final-pr-review.json').write_text(json.dumps(review,indent=2)+'\n')
text=(r/'artifacts/resource-compact-final-pr-comment-base.md').read_text()
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));(r/'artifacts/resource-compact-final-pr-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2)+'\n');t=u['totals'];text+=f"\nAI資源量（コメント直前ローカル累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数二重加算なし。\n"
p=r/'artifacts/resource-compact-final-pr-comment.md';p.write_text(text);done=subprocess.run(['gh','pr','comment','116','--repo',repo,'--body-file',str(p)],capture_output=True,text=True);(r/'artifacts/resource-compact-final-pr-comment.log').write_text(done.stdout+done.stderr);assert done.returncode==0;print(done.stdout)
subprocess.run(['gh','pr','merge','116','--repo',repo,'--merge','--match-head-commit',head],check=True)
print(subprocess.check_output(['gh','pr','view','116','--repo',repo,'--json','state,mergedAt,mergeCommit'],text=True))
