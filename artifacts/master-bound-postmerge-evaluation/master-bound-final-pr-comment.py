from pathlib import Path
import json,subprocess
r=Path.cwd();pr=json.loads((r/'artifacts/master-bound-final-pr-review.json').read_text());head=pr['headRefOid'];text=Path('/tmp/master-bound-pr-body.md').read_text()+f'''\n同CPU45・10秒・1thread・16GiBでの比較（HiGHS1.16.0、SCIP10.0.2、GLPK5.0、主解/下界）:

| 問題 | Rust | HiGHS | SCIP | GLPK |
|---|---|---|---|---|
| savsched1 | なし / 26.49999999997751 | 81005.5 / 58.79999999998108 | なし / −801160.2999999773 | なし / なし |
| map10 | なし / −877.0805141714454 | −204 / −586.1927440656348 | −205 / −589.7860776579956 | なし / なし |
| jarama | なし / 285.9999999999931 | なし / 0 | なし / 0 | なし / なし |

最終60秒/nativeの3整数点と9 Rust出力の証明検査が全有効。188の376件は各出力のカット/境界等の証明検査であり、376件の最適性証明という意味ではありません。最適件数は両側1です。Binkar主解改善とsp97下界改善、seymour/sp98下界悪化は全て新処理呼出0で、効果へ帰属しません。11resource→11timeも成功件数へ加えません。

[確定結果](https://github.com/iwiwi/mip-codex/blob/{head}/docs/MASTER_BOUND_REPORT_RESULTS.md)。最終14696ファイル304358550 bytes→46209956 bytes、未採用の大きい双対/名目/代替点等19846ファイル144979068 bytes→27590630 bytes、RENS strict10751/追加21395原ファイルも全payload SHAを展開照合して保存しました。ELF/生成LPは除外SHA台帳とローカル原本を保持。監査のimport設定失敗と再試行失敗も残し、ソルバー誤証明として混同していません。

原行LPと表示専用の根下界だけを採用し、名目基底/代替点/RENSコードは含めません。全240/3600秒の自作キャンペーンは未開始、11月19日UTC目安を維持。総合優位・新記録は未達成です。\n'''
u=json.loads(subprocess.check_output(['python3','scripts/ai_usage.py'],text=True));(r/'artifacts/master-bound-final-pr-ai-usage.json').write_text(json.dumps(u,ensure_ascii=False,indent=2)+'\n');t=u['totals'];text+=f"\nAI資源量（コメント直前ローカル累積、{u['sessions']} sessions）: total {t['total_tokens']:,}, input {t['input_tokens']:,}（cached {t['cached_input_tokens']:,}を含む）, output {t['output_tokens']:,}（reasoning {t['reasoning_output_tokens']:,}を含む）。内数は二重加算なし。\n";p=r/'artifacts/master-bound-final-pr-comment.md';p.write_text(text);a=subprocess.run(['gh','pr','comment',str(pr['number']),'--repo','iwiwi/mip-codex','--body-file',str(p)],capture_output=True,text=True);(r/'artifacts/master-bound-final-pr-comment.log').write_text(a.stdout+a.stderr);print(a.stdout,a.stderr);assert a.returncode==0
