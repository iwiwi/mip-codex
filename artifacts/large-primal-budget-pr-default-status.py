from pathlib import Path
import subprocess,json
p=Path('artifacts/large-primal-budget-pr-body.md');body=p.read_text();body='現headはcore939230aa1d・FP6021621d57155bb7のdefault ON候補へ更新し、再検証中です（draft・未統合）。新規353Rust/1200oracle/7410改ざん拒否/12largeのGMP・証明検査は成功、188問と実入力60/300秒の再測定は進行中。以下の成績は明記した初期defaultOFF源515fd/e180の明示0/1測定です。\n\n'+body
request=Path('artifacts/large-primal-budget-pr-default-status.json');request.write_text(json.dumps(dict(body=body),ensure_ascii=False));r=subprocess.run(['gh','api','--method','PATCH','repos/iwiwi/mip-codex/pulls/129','--input',str(request)],capture_output=True,text=True);Path('artifacts/large-primal-budget-pr-default-status.log').write_text(r.stdout+r.stderr);r.check_returncode();print('PR129 body updated: defaultON revalidation in progress')
