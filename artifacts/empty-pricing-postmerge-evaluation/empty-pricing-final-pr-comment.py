from pathlib import Path
import json,subprocess
r=Path.cwd();repo='iwiwi/mip-codex';head='8567771328ec4f5e67aa7cb2d3a6358a1ea69b13'
a=json.loads((r/'artifacts/empty-pricing-default-development-audit/summary.json').read_text());assert a['completed']==a['cut_valid']==376 and a['points']==a['strict_valid']==a['official_valid']==128
archive=json.loads(Path('/home/ubuntu/mip-priced-repair/artifacts/empty-pricing-final-evaluation/archive-summary.json').read_text());assert archive['files']==archive['verified_payloads']==13717
review=json.loads(subprocess.check_output(['gh','pr','view','115','--repo',repo,'--json','headRefOid,baseRefName,isDraft,mergeable,statusCheckRollup'],text=True));assert review['headRefOid']==head and review['baseRefName']=='main'and not review['isDraft']and not review['statusCheckRollup'];assert review['mergeable']=='MERGEABLE',review
(r/'artifacts/empty-pricing-final-pr-review.json').write_text(json.dumps(review,indent=2)+'\n')
text=(r/'artifacts/empty-pricing-final-pr-comment-base.md').read_text()+f"\n原記録{archive['files']}ファイル/{archive['original_bytes']} bytes→{archive['bundled_bytes']} bytes、全{archive['verified_payloads']}payload SHA照合、ELF/生成LP{archive['excluded']}本は除外台帳とともにローカル保持。\n"
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));(r/'artifacts/empty-pricing-final-pr-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2)+'\n');t=u['totals'];text+=f"\nAI資源量（コメント直前ローカル累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数二重加算なし。\n"
p=r/'artifacts/empty-pricing-final-pr-comment.md';p.write_text(text);done=subprocess.run(['gh','pr','comment','115','--repo',repo,'--body-file',str(p)],capture_output=True,text=True);(r/'artifacts/empty-pricing-final-pr-comment.log').write_text(done.stdout+done.stderr);assert done.returncode==0;print(done.stdout)
subprocess.run(['gh','pr','merge','115','--repo',repo,'--merge','--match-head-commit',head],check=True)
print(subprocess.check_output(['gh','pr','view','115','--repo',repo,'--json','state,mergedAt,mergeCommit'],text=True))
