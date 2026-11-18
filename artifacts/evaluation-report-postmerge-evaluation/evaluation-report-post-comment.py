from pathlib import Path
import subprocess,json,shutil
root=Path.cwd();out=root/'artifacts/evaluation-report-main-audit';source=Path('/home/ubuntu/mip-evaluation-report/artifacts/evaluation-report-tests')
for name in ['adopt.log','ai-usage.json']:shutil.copy2(source/name,out/('adoption-'+name))
receipt=json.load(open(out/'receipt.json'));assert receipt['new_main']=='6b8214962378da04af503ffb7544c85b00da83fd'
usage=subprocess.check_output(['python3','scripts/ai_usage.py'],text=True);(out/'post-ai-usage.json').write_text(usage)
body='main 6b821496へFF更新し、人工データ8テストを再度確認しました。Rust/測定/検査コードは前main f05f2944と同一、本体CLI SHA256 0bf00779798a657b02581a960261473b3093a21aae08d206b0d180f328a19876も不変です。最終240問config46ffdbd56ef5e38d1a493a96b39f23bc0333dd4c08b2960047dcbf836c9609e9はcreate-onlyのまま保持します。検証群の個別成績は読みませんでした。\n\nAI資源量（コメント直前再集計）：\n```json\n'+usage.strip()+'\n```\n'
p=out/'post-comment.txt';p.write_text(body);subprocess.run(['gh','pr','comment','133','--repo','iwiwi/mip-codex','--body-file',str(p)],check=True)
