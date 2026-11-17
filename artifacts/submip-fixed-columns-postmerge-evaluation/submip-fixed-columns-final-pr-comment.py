from pathlib import Path
import json,subprocess
r=Path.cwd();text=Path('/tmp/submip-fixed-columns-pr-body.md').read_text()+'''\n同CPU/10秒/1thread/16GiBのnative比較（主解/下界）:

| 問題 | Rust | HiGHS | SCIP | GLPK |
|---|---|---|---|---|
| blp-ic98 | 9913.0147792 / 4331.16835259805 | 4849.8189056 / 4440.873019913963 | 6215.1387872 / 4390.904799757216 | 10055.445048 / 4422.523743078512 |
| nu25-pr12 | 54360 / 53311.54372334667 | 53905 / 53905 最適 | 53905 / 53905 最適 | 53925 / 53700 |

新指紋の60秒/nativeは12点6Rust証明54カット全有効、seymour60秒2点2証明有効。seymourは主解460同値、下界400.2035334546947→400.1121189060993、子固定列除去0の時限差を保持。旧親8問試作と全失敗も保存。4324原ファイル248430903 bytesを2bundle32562039 bytesへ圧縮し、全payload SHAを展開照合した。ELF4本は除外SHA台帳とローカル原本を保持。全体優位・新記録は未達成。\n'''
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));(r/'artifacts/submip-fixed-columns-final-pr-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2)+'\n');t=u['totals'];text+=f"\nAI資源量（コメント直前ローカル累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数は二重加算なし。\n";p=r/'artifacts/submip-fixed-columns-final-pr-comment.md';p.write_text(text);a=subprocess.run(['gh','pr','comment','112','--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);(r/'artifacts/submip-fixed-columns-final-pr-comment.log').write_text(a.stdout+a.stderr);print(a.stdout,a.stderr);assert a.returncode==0
